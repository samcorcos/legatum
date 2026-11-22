#!/usr/bin/env python3
"""Train the learned value network (AI war plan step 3) and export integer weights for Rust.

Reads the fixed-width rows written by `crates/sim/examples/value_data.rs` (one or more run
directories, each with value-schema.json and value-rows-*.bin), trains a per-team MLP whose
scores feed the same estimate the AI computes (`legatum_ai::winchance` with gene
`learned_value`):

    p(team) = hold_share(team) + rest * softmax_over_teams(score(team) + race if team is me)

against the eventual winner, on whole games split by seed. It also trains a linear baseline on
the same inputs, then reports held-out calibration (Brier, log loss) and how often each
estimate ranks the eventual winner's seat first, against the hand formula Hard uses today.

The network is quantized (weights x WEIGHT_SCALE, biases in milli-units) and scored here with
the exact integer arithmetic of `value_net::Net::score` (i64, truncating division); the
exported file carries golden vectors that a Rust unit test checks.

  python3 tools/sim/train-value.py --data RUN_DIR [--data RUN_DIR2] \
      --out crates/ai/data/value_net.json --report docs/sim-results/value-net/fit.json

Requirements: tools/sim/requirements-value.txt (PyTorch, NumPy). Training only consumes the
engine's output; the rules stay in Rust.
"""
import argparse
import glob
import gzip
import json
import os
import sys
import time

import numpy as np
import torch
from torch import nn

WEIGHT_SCALE = 4096
ACTIVATION_LIMIT = 1_000_000
INPUTS_VERSION = 2


def load(dirs):
    """Every value-rows-*.bin under each directory, next to its value-schema.json[.gz] (a
    run-batch result has one per slice)."""
    schema = None
    parts = []
    for d in dirs:
        # run-batch gzips every *.json in a server's output, the schema included.
        schemas = sorted(glob.glob(os.path.join(d, "**", "value-schema.json*"), recursive=True))
        if not schemas:
            sys.exit(f"{d}: no value-schema.json")
        for sp in schemas:
            with (gzip.open(sp, "rt") if sp.endswith(".gz") else open(sp)) as f:
                s = json.load(f)
            if schema is None:
                schema = s
            elif s != schema:
                sys.exit(f"{sp}: schema differs from the first one")
            dtype = np.dtype([tuple(f[:2]) + ((tuple(f[2]),) if len(f) > 2 else ()) for f in s["dtype"]])
            for path in sorted(glob.glob(os.path.join(os.path.dirname(sp), "value-rows-*.bin"))):
                parts.append(np.fromfile(path, dtype=dtype))
    if schema is None or not parts:
        sys.exit("no rows found")
    if schema["inputs_version"] != INPUTS_VERSION:
        sys.exit(f"rows carry inputs v{schema['inputs_version']}, trainer expects v{INPUTS_VERSION}")
    rows = np.concatenate(parts)
    # Stable order: whatever order the parallel games finished in.
    rows = rows[np.lexsort((rows["seat"], rows["turn"], rows["seed"]))]
    return schema, rows


def tensors(rows):
    n = rows["n_teams"].astype(np.int64)
    teams = rows["x"].shape[1]
    mask = np.arange(teams)[None, :] < n[:, None]
    race = np.zeros(mask.shape, dtype=np.float32)
    race[np.arange(len(rows)), rows["me"]] = rows["race"] / 1000.0
    return {
        "x": torch.tensor(rows["x"].astype(np.float32) / 1000.0),
        "mask": torch.tensor(mask),
        "race": torch.tensor(race),
        "hold": torch.tensor(rows["hold"].astype(np.float32) / 1000.0),
        "rest": torch.tensor(rows["rest"].astype(np.float32) / 1000.0),
        "me": torch.tensor(rows["me"].astype(np.int64)),
        "winner": torch.tensor(rows["winner"].astype(np.int64)),
    }


def team_probs(scores, t):
    """Each team's win probability: hold share plus the rest shared by the logit."""
    s = scores + t["race"]
    s = s.masked_fill(~t["mask"], -1e9)
    soft = torch.softmax(s, dim=1)
    return t["hold"] + t["rest"][:, None] * soft


class Refit(nn.Module):
    """The hand formula itself: no bias, only the nine linear features (`winchance::FEATURES`)."""

    def __init__(self):
        super().__init__()
        self.l = nn.Linear(9, 1, bias=False)

    def forward(self, x):
        return self.l(x[:, :9])


def make_model(kind, inputs, hidden):
    if kind == "refit":
        return Refit()
    if kind == "linear":
        return nn.Linear(inputs, 1)
    layers, width = [], inputs
    for h in hidden:
        layers += [nn.Linear(width, h), nn.ReLU()]
        width = h
    layers.append(nn.Linear(width, 1))
    return nn.Sequential(*layers)


def scores_of(model, x):
    b, k, f = x.shape
    return model(x.reshape(b * k, f)).reshape(b, k)


def nll(model, t, idx):
    sub = {k: v[idx] for k, v in t.items()}
    p = team_probs(scores_of(model, sub["x"]), sub)
    pw = p.gather(1, sub["winner"][:, None]).squeeze(1)
    return -torch.log(pw.clamp_min(1e-6)).mean()


def train(kind, t, train_idx, val_idx, hidden, epochs, seed, lr, wd, batch):
    torch.manual_seed(seed)
    model = make_model(kind, t["x"].shape[2], hidden)
    opt = torch.optim.AdamW(model.parameters(), lr=lr, weight_decay=wd)
    best, best_state, stale = float("inf"), None, 0
    g = torch.Generator().manual_seed(seed)
    for epoch in range(epochs):
        model.train()
        perm = train_idx[torch.randperm(len(train_idx), generator=g)]
        for i in range(0, len(perm), batch):
            opt.zero_grad()
            loss = nll(model, t, perm[i : i + batch])
            loss.backward()
            opt.step()
        model.eval()
        with torch.no_grad():
            v = nll(model, t, val_idx).item()
        if v < best - 1e-5:
            best, stale = v, 0
            best_state = {k: x.clone() for k, x in model.state_dict().items()}
        else:
            stale += 1
        print(f"  {kind} epoch {epoch + 1}: validation log loss {v:.4f}", file=sys.stderr)
        if stale >= 3:
            break
    model.load_state_dict(best_state)
    return model


def linear_layers(model):
    mods = [model] if isinstance(model, nn.Linear) else [m for m in model if isinstance(m, nn.Linear)]
    out = []
    for i, m in enumerate(mods):
        w = m.weight.detach().numpy().astype(np.float64)
        b = m.bias.detach().numpy().astype(np.float64)
        out.append(
            {
                "in": int(w.shape[1]),
                "out": int(w.shape[0]),
                "w": np.rint(w * WEIGHT_SCALE).astype(np.int64).tolist(),
                "b": np.rint(b * 1000).astype(np.int64).tolist(),
                "relu": i < len(mods) - 1,
            }
        )
    return out


def div_trunc(a, s):
    return np.where(a >= 0, a // s, -((-a) // s))


def int_scores(layers, x):
    """`value_net::Net::score` for every row of x (int inputs, any leading shape)."""
    shape = x.shape[:-1]
    a = x.reshape(-1, x.shape[-1]).astype(np.int64)
    for l in layers:
        w = np.array(l["w"], dtype=np.int64)
        b = np.array(l["b"], dtype=np.int64)
        z = div_trunc(a @ w.T, WEIGHT_SCALE) + b
        if l["relu"]:
            z = np.maximum(z, 0)
        a = np.clip(z, -ACTIVATION_LIMIT, ACTIVATION_LIMIT)
    return a[:, 0].reshape(shape)


def own_from_scores(scores_milli, rows):
    """This seat's chance from integer team scores (milli-logits), as the AI computes it."""
    n = rows["n_teams"].astype(np.int64)
    k = rows["x"].shape[1]
    mask = np.arange(k)[None, :] < n[:, None]
    s = scores_milli.astype(np.float64) / 1000.0
    s[np.arange(len(rows)), rows["me"]] += rows["race"] / 1000.0
    s = np.where(mask, s, -1e9)
    s -= s.max(axis=1, keepdims=True)
    e = np.exp(s) * mask
    soft = e / e.sum(axis=1, keepdims=True)
    me = rows["me"].astype(np.int64)
    idx = np.arange(len(rows))
    return rows["hold"][idx, me] / 1000.0 + rows["rest"] / 1000.0 * soft[idx, me]


def metrics(p, rows):
    y = (rows["me"] == rows["winner"]).astype(np.float64)
    p = np.clip(p, 1e-6, 1 - 1e-6)
    uniform = 1.0 / rows["n_teams"]
    brier = float(np.mean((p - y) ** 2))
    base = float(np.mean((uniform - y) ** 2))
    out = {
        "rows": int(len(rows)),
        "brier": round(brier, 5),
        "brier_skill_vs_uniform": round(1 - brier / base, 4),
        "log_loss": round(float(np.mean(-(y * np.log(p) + (1 - y) * np.log(1 - p)))), 5),
    }
    # Picks the winner: at each (game, turn), does the highest own estimate belong to the
    # eventual winner's seat?
    seed, turn = rows["seed"], rows["turn"]
    order = np.lexsort((-p, turn, seed))
    first = np.ones(len(order), dtype=bool)
    first[1:] = (seed[order][1:] != seed[order][:-1]) | (turn[order][1:] != turn[order][:-1])
    top = order[first]
    out["picks_winner"] = round(float(np.mean(y[top])), 4)
    out["positions"] = int(len(top))
    return out


def by_phase(p, rows):
    out = {}
    for name, lo, hi in [("T1-60", 0, 61), ("T61-120", 61, 121), ("T121-200", 121, 201), ("T201+", 201, 10_000)]:
        m = (rows["turn"] >= lo) & (rows["turn"] < hi)
        if m.sum() > 0:
            out[name] = metrics(p[m], rows[m])
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--data", action="append", default=[], help="run directory (repeatable)")
    ap.add_argument("--data-list", help="file listing run directories, one per line (tools/sim/value-generate.sh)")
    ap.add_argument("--out", help="weights file to write (crates/ai/data/value_net.json)")
    ap.add_argument("--report", help="fit report JSON to write")
    ap.add_argument("--hidden", default="32,16")
    ap.add_argument("--epochs", type=int, default=40)
    ap.add_argument("--lr", type=float, default=3e-3)
    ap.add_argument("--weight-decay", type=float, default=1e-4)
    ap.add_argument("--batch", type=int, default=2048)
    ap.add_argument("--holdout-mod", type=int, default=5, help="seed %% N == 0 is held out")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--threads", type=int, default=0)
    ap.add_argument("--data-label", help="how the data is named in the outputs (default: the run directories' names)")
    args = ap.parse_args()
    if args.threads:
        torch.set_num_threads(args.threads)
    hidden = [int(h) for h in args.hidden.split(",") if h]
    if args.data_list:
        with open(args.data_list) as f:
            args.data += [l.strip() for l in f if l.strip()]
    args.data = list(dict.fromkeys(args.data))
    if not args.data:
        sys.exit("give --data or --data-list")
    data_label = args.data_label or ",".join(os.path.basename(os.path.normpath(d)) for d in args.data)
    started = time.time()
    schema, rows = load(args.data)
    seeds = rows["seed"].astype(np.uint64)
    held = (seeds % np.uint64(args.holdout_mod)) == 0
    # Validation for early stopping: another whole-game slice of the training seeds.
    val = (~held) & ((seeds // np.uint64(args.holdout_mod)) % np.uint64(10) == 0)
    fit = (~held) & (~val)
    games = lambda m: int(len(np.unique(seeds[m])))
    print(
        f"rows {len(rows)}: fit {fit.sum()} ({games(fit)} games), validation {val.sum()} "
        f"({games(val)}), held out {held.sum()} ({games(held)})",
        file=sys.stderr,
    )
    t = tensors(rows)
    fit_idx = torch.tensor(np.flatnonzero(fit))
    val_idx = torch.tensor(np.flatnonzero(val))
    common = dict(epochs=args.epochs, seed=args.seed, lr=args.lr, wd=args.weight_decay, batch=args.batch)
    refit = train("refit", t, fit_idx, val_idx, [], **common)
    refit_weights = np.rint(refit.l.weight.detach().numpy()[0].astype(np.float64) * 1000).astype(np.int64)
    linear = train("linear", t, fit_idx, val_idx, [], **common)
    net = train("mlp", t, fit_idx, val_idx, hidden, **common)
    layers = linear_layers(net)
    lin_layers = linear_layers(linear)

    h = rows[held]
    with torch.no_grad():
        th = {k: v[torch.tensor(np.flatnonzero(held))] for k, v in t.items()}
        me = torch.tensor(h["me"].astype(np.int64))[:, None]
        p_lin = team_probs(scores_of(linear, th["x"]), th).gather(1, me).squeeze(1).numpy()
        p_net = team_probs(scores_of(net, th["x"]), th).gather(1, me).squeeze(1).numpy()
    p_int = own_from_scores(int_scores(layers, h["x"]), h)
    p_lin_int = own_from_scores(int_scores(lin_layers, h["x"]), h)
    # `winchance::estimate_with` with the refitted weights: sum(x * w) / 1000 milli-logits.
    refit_scores = div_trunc((h["x"][:, :, :9].astype(np.int64) * refit_weights).sum(axis=2), 1000)
    p_refit_int = own_from_scores(refit_scores, h)
    estimates = {
        "uniform": 1.0 / h["n_teams"],
        "hand (Hard's plan today)": h["hand"] / 10000.0,
        "fitted linear (search rollouts)": h["fitted"] / 10000.0,
        "hand formula refitted (learned_value=2)": p_refit_int,
        "linear baseline, float": p_lin,
        "linear baseline, integer": p_lin_int,
        "network, float": p_net,
        "network, integer (what Rust runs)": p_int,
    }
    report = {
        "date": time.strftime("%Y-%m-%d"),
        "data": data_label,
        "rows": int(len(rows)),
        "games": {"fit": games(fit), "validation": games(val), "held_out": games(held)},
        "hidden": hidden,
        "parameters": int(sum(p.numel() for p in net.parameters())),
        "refit_weights": refit_weights.tolist(),
        "held_out": {name: metrics(p, h) for name, p in estimates.items()},
        "held_out_by_phase": {
            name: by_phase(p, h)
            for name, p in estimates.items()
            if name in ("hand (Hard's plan today)", "hand formula refitted (learned_value=2)", "network, integer (what Rust runs)")
        },
        "held_out_by_players": {
            str(n): {
                name: metrics(p[h["players"] == n], h[h["players"] == n])
                for name, p in estimates.items()
                if name in ("uniform", "hand (Hard's plan today)", "hand formula refitted (learned_value=2)", "network, integer (what Rust runs)")
            }
            for n in sorted(set(h["players"].tolist()))
        },
        "train_seconds": round(time.time() - started, 1),
    }
    for name, m in report["held_out"].items():
        print(
            f"{name:>36}: Brier {m['brier']:.4f} (skill {m['brier_skill_vs_uniform']:+.1%}), "
            f"log loss {m['log_loss']:.4f}, picks the winner {m['picks_winner']:.1%}",
            file=sys.stderr,
        )
    rng = np.random.default_rng(args.seed)
    team_rows = h["x"][h["n_teams"][:, None] > np.arange(h["x"].shape[1])[None, :]]
    pick = rng.choice(len(team_rows), size=min(8, len(team_rows)), replace=False)
    golden = [{"x": team_rows[i].astype(int).tolist(), "score": int(int_scores(layers, team_rows[i][None, :])[0])} for i in pick]
    if args.out:
        out = {
            "version": INPUTS_VERSION,
            "weight_scale": WEIGHT_SCALE,
            "input_names": schema["input_names"],
            "layers": layers,
            "refit_weights": refit_weights.tolist(),
            "meta": {
                "trained": report["date"],
                "data": data_label,
                "games": report["games"],
                "rows": report["rows"],
                "hidden": hidden,
                "held_out_network": report["held_out"]["network, integer (what Rust runs)"],
                "held_out_hand": report["held_out"]["hand (Hard's plan today)"],
                "held_out_refit": report["held_out"]["hand formula refitted (learned_value=2)"],
            },
            "golden": golden,
        }
        with open(args.out, "w") as f:
            json.dump(out, f, separators=(",", ":"))
            f.write("\n")
    if args.report:
        os.makedirs(os.path.dirname(args.report) or ".", exist_ok=True)
        with open(args.report, "w") as f:
            json.dump(report, f, indent=1)
            f.write("\n")


if __name__ == "__main__":
    main()
