#!/usr/bin/env node
// Storyboard: decide what story the run tells, then write the narration and the scene list.
//   node tools/film/storyboard.mjs <run-dir> [--out <dir>] [--fresh] [--no-llm] [--target-seconds N]
//
// Story selection is a creative step: Astra (gpt-6-astra via `codex exec`, read-only) gets every
// fact from the run log, proposes 2-3 narrative angles, picks the strongest with a rationale, and
// composes scenes from the renderer's building blocks in whatever order and length suit the
// material. Truth is not negotiable: every number in the result is checked against the log
// (lib/validate.mjs) and unsupported sentences are dropped. If the model is unavailable or its
// script does not survive validation, a deterministic template tells the story instead.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { loadRun, facts as factsOf } from "./lib/runlog.mjs";
import { validateScript } from "./lib/validate.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../..");
const pct = (x) => `${Math.round(x * 100)}%`;
const pct1 = (x) => `${(x * 100).toFixed(1)}%`;
const human = (s) => String(s || "").replaceAll("_", " ");
const BEHAVIOR_WORDS = {
  unanswered_attack: "units that take hits without answering",
  scout_not_fleeing: "scouts that stay next to enemies",
  garrison_under_ranged_fire: "garrisons that sit under ranged fire",
  idle_military_threatened_city: "healthy units idling near a threatened city",
  repeated_bounce: "moves that bounce off turn after turn",
  movement_oscillation: "units pacing back and forth",
  gold_hoarded_under_attack: "gold hoarded while under attack",
  settler_wandering: "settlers wandering without founding a city",
  research_idle: "empty research queues",
  production_idle: "cities with nothing in production",
};
const behaviorWords = (b) => BEHAVIOR_WORDS[b] || human(b);

// ---------- facts ----------
export function storyFacts(data) {
  const f = factsOf(data);
  const s = f.generations.series;
  const derived = {};
  if (s.length) {
    const pw = s.filter((g) => g.proxy_win_rate != null);
    if (pw.length) {
      const lo = pw.reduce((a, g) => (g.proxy_win_rate < a.proxy_win_rate ? g : a), pw[0]);
      const hi = pw.reduce((a, g) => (g.proxy_win_rate > a.proxy_win_rate ? g : a), pw[0]);
      derived.proxy_win_rate = { first: pw[0].proxy_win_rate, last: pw.at(-1).proxy_win_rate, lowest: { gen: lo.gen, value: lo.proxy_win_rate }, highest: { gen: hi.gen, value: hi.proxy_win_rate }, change_points: Math.round((pw.at(-1).proxy_win_rate - pw[0].proxy_win_rate) * 100) };
    }
  }
  if (f.before_after?.proxy_win_rate?.every((v) => v != null)) derived.checkpoint_proxy_change_points = Math.round((f.before_after.proxy_win_rate[1] - f.before_after.proxy_win_rate[0]) * 100);
  derived.eras = f.eras.length;
  derived.coach_observations = f.coach.reduce((a, c) => a + c.observations.length, 0);
  // Word forms of incident behaviors so narration can use plain language.
  derived.behavior_words = BEHAVIOR_WORDS;
  return { ...f, derived };
}

// ---------- the LLM step ----------
const SCENE_DOC = `Scene building blocks (compose any of them, in any order, any number of times):
- title: opening card. heading = film title, body = one-line subtitle.
- text: a calm text card. kicker (small caps line), heading, body (1-3 short lines). Use for context, questions, or what's unsolved.
- population: one small rounded blob creature per candidate, colored by its strategy archetype; the generation counter advances from gen_from to gen_to and blobs change color as strategies spread. archetype = optionally highlight one archetype (others dim). kicker optional.
- graph: a graph that draws itself left to right over generations gen_from..gen_to. metric = "archetype_share" (stacked area of which strategies the population uses), "proxy_win_rate", "best_fitness" or "mean_fitness". archetype = optional highlight for archetype_share. heading optional. Stand-in stage boundaries and takeover/extinction events are marked automatically.
- map: a real showcase game replayed on the hex map (territory, cities, armies). game_id = a showcase game id; turn_from..turn_to = turns to play through (the camera then settles on turn_to); focus_player = seat index (0-based, the "player" of a highlight turn) to zoom toward and ring in gold, or null for the whole map; zoom = 1.5-3 (default 2.2). kicker/heading optional (heading shows top-left).
- zoom: same as map but meant for one decision: set turn_from = turn_to = the highlight turn, focus_player, zoom 2.5-3.
- versus: two showcase games side by side at the same turn range. game_id (left), game_id_b (right), label_a, label_b, turn_from, turn_to, focus_player / focus_player_b. Best for the same seed before and after.
- matrix: the matchup matrix at checkpoint_gen (who finishes ahead of whom).
- stat: big before -> after numbers for the Proxy win rate and AI survival at turn 150, first vs last checkpoint (numbers are filled in from the log).
- end: closing card; credits are added automatically. heading optional.`;

function prompt(facts, targetSeconds) {
  const words = Math.round(targetSeconds * 2.5);
  return `You are the storyteller for a short narrated film about an overnight run that tunes the AI of Legatum, a Civ-6-like sci-fi 4X game. The viewer is the game's designer, coming back in the morning to see what the AI learned. The film's look is inspired by Primer (Justin Helps): calm narrator, simple rounded blob creatures for populations, graphs that draw themselves, one idea per scene, each idea shown once and clearly.

YOUR JOB
1. Read the FACTS below (everything the run logged). Find what actually happened: the arcs in the curves, which strategies spread or vanished and when, what the coach observed, which showcase games illustrate it, what is still unsolved.
2. Propose 2 or 3 genuinely different narrative ANGLES (for example a single arc "X appeared and was countered by Y", a mystery "why did Z die out at generation N?", a before/after rivalry, a few vignettes). For each: a title, a one-sentence logline, the structure, and why it fits this material.
3. Pick the strongest angle (chosen = its index) and explain why in rationale (2-4 sentences).
4. Write the film for that angle: a title, a subtitle, and the scene list. Pacing: one idea per scene, 1-3 short spoken sentences per scene, plain words, present tense, calm. The length follows the material: aim for about ${targetSeconds} seconds of narration (about ${words} words in total), fewer if the material is thin. Start with a title scene and end with an end scene.

TRUTH RULES (a validator enforces them and deletes any sentence that breaks one):
- Every number you say or show must appear in FACTS (a generation from the log, a turn from a highlight or evidence, a rate from the log; percentages may be rounded to whole points). Write every quantity as digits ("38%", "generation 14", "turn 141"), never as words.
- Never invent causes. If FACTS do not say why something happened, say what happened, or ask it as an open question.
- If FACTS.standin is present, this is a STAND-IN rehearsal, not a real evolution: the "generations" are fixed difficulty stages of the existing AI, nothing was optimized, archetypes come from a stand-in classifier and the coach is a rule, not an LLM. Say so plainly early on, and do not claim the AI "learned" anything; describe what changed between the stages instead. Stand-in stages: ${JSON.stringify(facts.standin?.stages || null)}.
- Use behavior names in plain words (derived.behavior_words maps the detector's keys to phrases).
- Map, zoom and versus scenes may only use game ids listed under checkpoints[].showcase. Pick turns from their highlight_turns.
- Leave any field a scene does not use as null.

${SCENE_DOC}

FACTS (JSON):
${JSON.stringify(facts)}
`;
}

const SCHEMA = {
  type: "object",
  additionalProperties: false,
  required: ["angles", "chosen", "rationale", "title", "subtitle", "scenes"],
  properties: {
    angles: { type: "array", items: { type: "object", additionalProperties: false, required: ["title", "logline", "structure", "why"], properties: { title: { type: "string" }, logline: { type: "string" }, structure: { type: "string" }, why: { type: "string" } } } },
    chosen: { type: "integer" },
    rationale: { type: "string" },
    title: { type: "string" },
    subtitle: { type: "string" },
    scenes: {
      type: "array",
      items: {
        type: "object",
        additionalProperties: false,
        required: ["type", "narration", "kicker", "heading", "body", "gen_from", "gen_to", "metric", "archetype", "game_id", "game_id_b", "turn_from", "turn_to", "focus_player", "focus_player_b", "zoom", "label_a", "label_b", "checkpoint_gen"],
        properties: {
          type: { type: "string", enum: ["title", "text", "population", "graph", "map", "zoom", "versus", "matrix", "stat", "end"] },
          narration: { type: "array", items: { type: "string" } },
          kicker: { type: ["string", "null"] }, heading: { type: ["string", "null"] }, body: { type: ["string", "null"] },
          gen_from: { type: ["integer", "null"] }, gen_to: { type: ["integer", "null"] },
          metric: { type: ["string", "null"], enum: ["archetype_share", "proxy_win_rate", "best_fitness", "mean_fitness", null] },
          archetype: { type: ["string", "null"] },
          game_id: { type: ["string", "null"] }, game_id_b: { type: ["string", "null"] },
          turn_from: { type: ["integer", "null"] }, turn_to: { type: ["integer", "null"] },
          focus_player: { type: ["integer", "null"] }, focus_player_b: { type: ["integer", "null"] },
          zoom: { type: ["number", "null"] },
          label_a: { type: ["string", "null"] }, label_b: { type: ["string", "null"] },
          checkpoint_gen: { type: ["integer", "null"] },
        },
      },
    },
  },
};

function fromLlm(raw) {
  const scenes = raw.scenes.map((s) => {
    const { type, narration, ...rest } = s;
    const params = Object.fromEntries(Object.entries(rest).filter(([, v]) => v != null && v !== ""));
    return { type, narration, params };
  });
  return { source: "llm", angles: raw.angles, chosen: raw.chosen, rationale: raw.rationale, title: raw.title, subtitle: raw.subtitle, scenes };
}

function runAstra(facts, targetSeconds, workDir) {
  const codex = process.env.LEGATUM_CODEX || "codex";
  const schemaPath = path.join(workDir, "storyboard.schema.json");
  const outPath = path.join(workDir, "storyboard.llm.json");
  fs.writeFileSync(schemaPath, JSON.stringify(SCHEMA));
  const text = prompt(facts, targetSeconds);
  const promptFile = path.join(workDir, "storyboard.prompt.txt");
  // Resumable: the same facts and prompt reuse the model's last answer instead of asking again.
  if (fs.existsSync(outPath) && fs.existsSync(promptFile) && fs.readFileSync(promptFile, "utf8") === text) {
    try { return fromLlm(JSON.parse(fs.readFileSync(outPath, "utf8"))); } catch { /* ask again */ }
  }
  fs.writeFileSync(promptFile, text);
  if (fs.existsSync(outPath)) fs.renameSync(outPath, `${outPath}.previous`);
  const r = spawnSync(codex, ["exec", "-m", process.env.LEGATUM_FILM_MODEL || "gpt-6-astra", "-s", "read-only", "-C", root, "--output-schema", schemaPath, "-o", outPath, "-"], { input: text, encoding: "utf8", timeout: 20 * 60e3, maxBuffer: 1 << 26 });
  fs.writeFileSync(path.join(workDir, "storyboard.codex.log"), `${r.stdout || ""}\n${r.stderr || ""}`);
  if (r.status !== 0 || !fs.existsSync(outPath)) throw new Error(`codex exec failed (status ${r.status}${r.error ? `, ${r.error.message}` : ""})`);
  return fromLlm(JSON.parse(fs.readFileSync(outPath, "utf8")));
}

// ---------- the deterministic fallback ----------
export function templateScript(facts) {
  const s = facts.generations.series, first = s[0], last = s.at(-1);
  const cps = facts.checkpoints, cpFirst = cps[0], cpLast = cps.at(-1);
  const scenes = [];
  const add = (type, narration, params = {}) => scenes.push({ type, narration, params });
  const stand = facts.standin;
  add("title", [], { heading: stand ? "A rehearsal run" : "What the AI learned", body: `Evolution run ${facts.run_id}` });
  if (stand) {
    add("text", [
      "This is a rehearsal, not an evolution.",
      `Each generation is one batch of real games, and the AI steps through fixed stages: ${stand.stages.map((x) => `${x.name} from generation ${x.from_gen} to ${x.to_gen}`).join(", then ")}.`,
      "Nothing is optimized. We only watch what changes.",
    ], { kicker: "Stand-in", heading: "A rehearsal with real games", body: stand.stages.map((x) => `Generations ${x.from_gen}–${x.to_gen}: ${x.name}`).join("\n") });
  }
  const topStart = Object.entries(first.archetype_share).sort((a, b) => b[1] - a[1])[0];
  add("population", [
    `At generation ${first.gen}, each blob is one AI in one game, colored by the strategy it leaned into.`,
    `The most common was ${topStart[0]}, at ${pct(topStart[1])}.`,
  ], { gen_from: first.gen, gen_to: first.gen, kicker: "The starting AI" });
  const hasTurns = (s) => s?.highlight_turns?.length > 0;
  const cp0 = cpFirst?.showcase?.find(hasTurns);
  if (cp0) {
    const h = cp0.highlight_turns[0], end = cp0.highlight_turns.at(-1);
    add("map", [`Here is a typical game from generation ${cpFirst.gen}.`, `By turn ${end.turn}, ${end.note.replace(/\s*\(seat \d+[^)]*\)/, "")}.`], { game_id: cp0.game_id, turn_from: Math.max(1, h.turn - 20), turn_to: end.turn, focus_player: end.player, kicker: `Generation ${cpFirst.gen}` });
  }
  add("population", [`Then the generations roll on, from ${first.gen} to ${last.gen}.`, "Watch the colors: each change is a strategy spreading or shrinking."], { gen_from: first.gen, gen_to: last.gen });
  add("graph", [`Here are the same shares as a graph.`, facts.eras.length > 1 ? `The lead changes hands ${facts.eras.length - 1} times, between ${[...new Set(facts.eras.map((e) => e.dominant))].join(", ")}.` : `${facts.eras[0].dominant} stays in front throughout.`], { metric: "archetype_share", gen_from: first.gen, gen_to: last.gen });
  for (const c of facts.coach) {
    const o = c.observations.find((x) => x.metric_change) || null;
    if (!o) continue;
    const mc = o.metric_change;
    const isRate = /incidents/.test(mc.name);
    const what = o.archetype ? `${o.archetype} play` : behaviorWords(o.behavior);
    const line = isRate ? `${what[0].toUpperCase()}${what.slice(1)} went from ${pct1(mc.before)} to ${pct1(mc.after)} of the chances to do it.` : `${what} went from ${pct(mc.before)} to ${pct(mc.after)} of the population.`;
    const ev = o.evidence[0];
    const cp = cps.find((x) => x.gen === c.gen);
    const show = [cp?.showcase?.find((x) => x.game_id === ev?.game_id), ...(cp?.showcase || [])].find(hasTurns);
    add("text", [`At generation ${c.gen}, the coach reviews the games.`, line], { kicker: `Coach · generation ${c.gen}`, heading: `${o.kind === "weakness" ? "A weakness" : o.kind === "disappeared" ? "A habit fades" : "Something new"}: ${what}`, body: line });
    if (show && show !== cp0) {
      const h = show.highlight_turns[0], end = show.highlight_turns.at(-1);
      const who = h.note.split(" (")[0], lean = /classified (\w+)/.exec(h.note)?.[1];
      add("map", [`In this game from generation ${c.gen}, watch ${who}${lean ? `, which the classifier calls ${lean}` : ""}.`], { game_id: show.game_id, turn_from: Math.max(1, h.turn - 15), turn_to: end.turn, focus_player: h.player, kicker: `Generation ${c.gen}` });
    }
  }
  const pw = facts.derived.proxy_win_rate;
  if (pw) add("graph", [`The Proxy, a scripted stand-in for a strong human, won ${pct(pw.first)} of games at generation ${first.gen}.`, `By generation ${last.gen} it won ${pct(pw.last)}, with a low of ${pct(pw.lowest.value)} at generation ${pw.lowest.gen}.`], { metric: "proxy_win_rate", gen_from: first.gen, gen_to: last.gen });
  const rematch = cpLast?.showcase?.find((x) => /Same seed/.test(x.why) && hasTurns(x));
  if (rematch && cp0) add("versus", [`Same map, same seed ${rematch.seed}: generation ${cpFirst.gen} on the left, generation ${cpLast.gen} on the right.`], { game_id: cp0.game_id, game_id_b: rematch.game_id, label_a: `Generation ${cpFirst.gen}`, label_b: `Generation ${cpLast.gen}`, turn_from: 1, turn_to: Math.min(cp0.highlight_turns.at(-1).turn, rematch.highlight_turns.at(-1).turn), focus_player: cp0.highlight_turns.at(-1).player, focus_player_b: rematch.highlight_turns.at(-1).player });
  if (facts.before_after) add("stat", [`Between checkpoint ${facts.before_after.before_gen} and checkpoint ${facts.before_after.after_gen}, the Proxy's win rate went from ${pct(facts.before_after.proxy_win_rate[0])} to ${pct(facts.before_after.proxy_win_rate[1])}.`], { heading: "The benchmark" });
  if (cpLast) add("matrix", [], { checkpoint_gen: cpLast.gen });
  const lastCoach = facts.coach.at(-1);
  const weak = lastCoach?.observations.filter((o) => o.kind === "weakness") || [];
  add("text", [
    weak.length ? `Still unsolved: ${weak.map((o) => behaviorWords(o.behavior)).join(", and ")}.` : "The last review found no new weakness.",
    stand ? "The real run will tune the AI's decisions, and this film will tell that story instead." : "The next run starts from here.",
  ], { kicker: "Still unsolved", heading: weak.length ? "What the AI still gets wrong" : "What's next", body: weak.map((o) => `${behaviorWords(o.behavior)}: ${pct1(o.metric_change.before)} → ${pct1(o.metric_change.after)}`).join("\n") });
  add("end", [], { heading: "Every number came from the run log" });
  return {
    source: "template",
    angles: [{ title: "The run in order", logline: "A walk through the run's stages, what the coach saw, and the before/after benchmark.", structure: "title, context, population, map, graphs, coach checkpoints, benchmark, unsolved", why: "Deterministic fallback when the storyteller model is unavailable." }],
    chosen: 0,
    rationale: "Template script: used because the LLM storyboard was unavailable or did not survive validation.",
    title: stand ? "A rehearsal run" : "What the AI learned",
    subtitle: `Evolution run ${facts.run_id}`,
    scenes,
  };
}

function ensureBookends(script) {
  const s = script.scenes;
  if (s[0]?.type !== "title") s.unshift({ type: "title", narration: [], params: { heading: script.title, body: script.subtitle } });
  if (s.at(-1)?.type !== "end") s.push({ type: "end", narration: [], params: {} });
  return script;
}

// ---------- main ----------
export async function storyboard(runDir, { out, fresh = false, llm = true, targetSeconds } = {}) {
  const data = loadRun(path.resolve(runDir));
  const facts = storyFacts(data);
  const mapGames = new Set(facts.checkpoints.flatMap((c) => c.showcase.map((s) => s.game_id)));
  const target = targetSeconds || (facts.standin || facts.generations.count < 40 ? 80 : Math.min(720, 300 + facts.coach.length * 25));
  const dir = out || path.join(process.env.LEGATUM_FILMS || path.join(os.homedir(), "LegatumData", "films"), "cache", data.id);
  fs.mkdirSync(dir, { recursive: true });
  const key = crypto.createHash("sha256").update(JSON.stringify({ facts, target, llm, v: 3 })).digest("hex").slice(0, 16);
  const file = path.join(dir, "script.json");
  if (!fresh && fs.existsSync(file)) {
    const cached = JSON.parse(fs.readFileSync(file, "utf8"));
    if (cached.facts_key === key) return cached;
  }
  let script = null, report = [], note = "";
  if (llm) {
    try {
      const raw = runAstra(facts, target, dir);
      const v = validateScript(raw, facts, { mapGames });
      const said = v.script.scenes.reduce((a, sc) => a + sc.narration.length, 0);
      if (v.script.scenes.length >= 4 && said >= 4) { script = v.script; report = v.report; }
      else note = `The model's script kept only ${v.script.scenes.length} scenes and ${said} sentences after validation; used the template. Validation notes: ${v.report.join(" | ")}`;
    } catch (e) {
      note = `Storyteller model unavailable: ${e.message}. Used the template.`;
    }
  }
  if (!script) {
    const v = validateScript(templateScript(facts), facts, { mapGames });
    script = v.script; report = v.report;
  }
  ensureBookends(script);
  const result = { ...script, run_id: data.id, run_dir: path.relative(root, data.dir), facts_key: key, target_seconds: target, validation: report, note, made_at: new Date().toISOString() };
  fs.writeFileSync(file, JSON.stringify(result, null, 2) + "\n");
  const md = [
    `# Film story: ${result.title}`,
    "",
    `Run \`${data.id}\`. Script source: **${result.source === "llm" ? "Astra (gpt-6-astra) storyteller" : "deterministic template"}**.${note ? ` ${note}` : ""}`,
    "",
    "## Angles considered",
    "",
    ...result.angles.map((a, i) => `${i + 1}. **${a.title}**${i === result.chosen ? " (chosen)" : ""}: ${a.logline}\n   - Structure: ${a.structure}\n   - Why: ${a.why}`),
    "",
    "## Why this one",
    "",
    result.rationale,
    "",
    "## Scenes",
    "",
    ...result.scenes.map((s, i) => `${i + 1}. \`${s.type}\` ${JSON.stringify(s.params)}${s.narration.length ? `\n   > ${s.narration.join(" ")}` : ""}`),
    "",
    "## Fact check",
    "",
    report.length ? report.map((r) => `- ${r}`).join("\n") : "Every number in the script was found in the run log; nothing was dropped.",
    "",
  ].join("\n");
  fs.writeFileSync(path.join(dir, "story.md"), md);
  return result;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const runDir = args.find((a) => !a.startsWith("--"));
  if (!runDir) { console.error("usage: node tools/film/storyboard.mjs <run-dir> [--out dir] [--fresh] [--no-llm] [--target-seconds N]"); process.exit(2); }
  const opt = (k) => { const i = args.indexOf(`--${k}`); return i >= 0 ? args[i + 1] : undefined; };
  const r = await storyboard(runDir, { out: opt("out"), fresh: args.includes("--fresh"), llm: !args.includes("--no-llm"), targetSeconds: opt("target-seconds") ? Number(opt("target-seconds")) : undefined });
  console.log(`${r.source} script: ${r.scenes.length} scenes, ${r.validation.length} validation notes`);
}
