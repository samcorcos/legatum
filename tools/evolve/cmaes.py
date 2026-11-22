"""A small, dependency-free CMA-ES (Hansen's (mu/mu_w, lambda)-CMA-ES, minimizing).

pycma and nevergrad are not installable here (Homebrew Python refuses `pip --user` under
PEP 668) and numpy is absent, so this implements the standard algorithm from Hansen's
"The CMA Evolution Strategy: A Tutorial" with plain lists. At ~40 genes the O(n^3) Jacobi
eigendecomposition per generation costs milliseconds next to minutes of simulation.

The whole state, including the random generator, round-trips through JSON so a run can
resume from any checkpoint and ask for exactly the same candidates again.
"""

from __future__ import annotations

import math
import random


def _jacobi_eigh(a: list[list[float]], sweeps: int = 60) -> tuple[list[float], list[list[float]]]:
    """Eigenvalues and eigenvectors (columns of V) of a symmetric matrix."""
    n = len(a)
    a = [row[:] for row in a]
    v = [[1.0 if i == j else 0.0 for j in range(n)] for i in range(n)]
    for _ in range(sweeps):
        off = sum(a[i][j] ** 2 for i in range(n) for j in range(i + 1, n))
        if off < 1e-22:
            break
        for p in range(n - 1):
            for q in range(p + 1, n):
                if abs(a[p][q]) < 1e-300:
                    continue
                theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q])
                t = (1.0 if theta >= 0 else -1.0) / (abs(theta) + math.sqrt(theta * theta + 1.0))
                c = 1.0 / math.sqrt(t * t + 1.0)
                s = t * c
                for k in range(n):
                    akp, akq = a[k][p], a[k][q]
                    a[k][p] = c * akp - s * akq
                    a[k][q] = s * akp + c * akq
                for k in range(n):
                    apk, aqk = a[p][k], a[q][k]
                    a[p][k] = c * apk - s * aqk
                    a[q][k] = s * apk + c * aqk
                for k in range(n):
                    vkp, vkq = v[k][p], v[k][q]
                    v[k][p] = c * vkp - s * vkq
                    v[k][q] = s * vkp + c * vkq
    return [a[i][i] for i in range(n)], v


class CMAES:
    def __init__(self, x0: list[float], sigma0: float, popsize: int | None = None, seed: int = 1):
        n = len(x0)
        if n < 1 or sigma0 <= 0:
            raise ValueError("CMA-ES needs at least one dimension and a positive sigma")
        self.n = n
        self.lam = popsize or 4 + int(3 * math.log(n))
        if self.lam < 2:
            raise ValueError("population must be at least 2")
        self.mu = self.lam // 2
        w = [math.log(self.mu + 0.5) - math.log(i + 1) for i in range(self.mu)]
        s = sum(w)
        self.weights = [x / s for x in w]
        self.mueff = 1.0 / sum(x * x for x in self.weights)
        self.cc = (4 + self.mueff / n) / (n + 4 + 2 * self.mueff / n)
        self.cs = (self.mueff + 2) / (n + self.mueff + 5)
        self.c1 = 2 / ((n + 1.3) ** 2 + self.mueff)
        self.cmu = min(1 - self.c1, 2 * (self.mueff - 2 + 1 / self.mueff) / ((n + 2) ** 2 + self.mueff))
        self.damps = 1 + 2 * max(0.0, math.sqrt((self.mueff - 1) / (n + 1)) - 1) + self.cs
        self.chin = math.sqrt(n) * (1 - 1 / (4 * n) + 1 / (21 * n * n))
        self.mean = list(map(float, x0))
        self.sigma = float(sigma0)
        self.pc = [0.0] * n
        self.ps = [0.0] * n
        self.C = [[1.0 if i == j else 0.0 for j in range(n)] for i in range(n)]
        self.B = [[1.0 if i == j else 0.0 for j in range(n)] for i in range(n)]
        self.D = [1.0] * n
        self.gen = 0
        self.rng = random.Random(seed)

    # ---- sampling ----
    def ask(self) -> list[list[float]]:
        out = []
        for _ in range(self.lam):
            z = [self.rng.gauss(0.0, 1.0) for _ in range(self.n)]
            dz = [self.D[i] * z[i] for i in range(self.n)]
            y = [sum(self.B[i][j] * dz[j] for j in range(self.n)) for i in range(self.n)]
            out.append([self.mean[i] + self.sigma * y[i] for i in range(self.n)])
        return out

    # ---- update (minimization) ----
    def tell(self, xs: list[list[float]], values: list[float]) -> None:
        if len(xs) != self.lam or len(values) != self.lam:
            raise ValueError(f"tell needs exactly {self.lam} candidates")
        n = self.n
        order = sorted(range(self.lam), key=lambda k: values[k])
        best = [xs[k] for k in order[: self.mu]]
        old = self.mean
        self.mean = [sum(self.weights[r] * best[r][i] for r in range(self.mu)) for i in range(n)]
        yw = [(self.mean[i] - old[i]) / self.sigma for i in range(n)]
        # C^{-1/2} yw = B D^-1 B^T yw
        bt = [sum(self.B[j][i] * yw[j] for j in range(n)) for i in range(n)]
        bt = [bt[i] / self.D[i] for i in range(n)]
        inv_sqrt = [sum(self.B[i][j] * bt[j] for j in range(n)) for i in range(n)]
        k = math.sqrt(self.cs * (2 - self.cs) * self.mueff)
        self.ps = [(1 - self.cs) * self.ps[i] + k * inv_sqrt[i] for i in range(n)]
        norm_ps = math.sqrt(sum(v * v for v in self.ps))
        hsig = norm_ps / math.sqrt(1 - (1 - self.cs) ** (2 * (self.gen + 1))) / self.chin < 1.4 + 2 / (n + 1)
        kc = math.sqrt(self.cc * (2 - self.cc) * self.mueff)
        self.pc = [(1 - self.cc) * self.pc[i] + (kc * yw[i] if hsig else 0.0) for i in range(n)]
        art = [[(x[i] - old[i]) / self.sigma for i in range(n)] for x in best]
        delta = (1 - hsig) * self.cc * (2 - self.cc)
        for i in range(n):
            for j in range(i + 1):
                rank_mu = sum(self.weights[r] * art[r][i] * art[r][j] for r in range(self.mu))
                v = (
                    (1 - self.c1 - self.cmu) * self.C[i][j]
                    + self.c1 * (self.pc[i] * self.pc[j] + delta * self.C[i][j])
                    + self.cmu * rank_mu
                )
                self.C[i][j] = self.C[j][i] = v
        self.sigma *= math.exp((self.cs / self.damps) * (norm_ps / self.chin - 1))
        self.sigma = min(self.sigma, 1e6)
        evals, vecs = _jacobi_eigh(self.C)
        self.D = [math.sqrt(max(e, 1e-20)) for e in evals]
        self.B = vecs
        self.gen += 1

    def stddevs(self) -> list[float]:
        """Per-coordinate standard deviation of the sampling distribution."""
        return [self.sigma * math.sqrt(max(self.C[i][i], 0.0)) for i in range(self.n)]

    # ---- persistence ----
    def to_dict(self) -> dict:
        version, internal, gauss_next = self.rng.getstate()
        return {
            "n": self.n, "lam": self.lam, "mean": self.mean, "sigma": self.sigma,
            "pc": self.pc, "ps": self.ps, "C": self.C, "B": self.B, "D": self.D, "gen": self.gen,
            "rng": [version, list(internal), gauss_next],
        }

    @classmethod
    def from_dict(cls, d: dict) -> "CMAES":
        es = cls(d["mean"], d["sigma"], d["lam"])
        for key in ("pc", "ps", "C", "B", "D", "gen"):
            setattr(es, key, d[key])
        version, internal, gauss_next = d["rng"]
        es.rng.setstate((version, tuple(internal), gauss_next))
        return es
