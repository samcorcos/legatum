"""Symmetric Nash solver, with an explicit unilateral-deviation certificate.

We try multiplicative replicator dynamics, then enumerate supports if it cycles.
For at most seven openings there are only 127 supports. Neither a stable-looking
trajectory nor a time average is called an equilibrium without checking regret.
"""

from itertools import combinations
import math


def regret(matrix, weights):
    payoffs = [sum(a * p for a, p in zip(row, weights)) for row in matrix]
    value = sum(p * u for p, u in zip(weights, payoffs))
    return max(0.0, max(payoffs) - value)


def solve_linear(matrix, rhs):
    a = [list(row) + [value] for row, value in zip(matrix, rhs)]
    n = len(a)
    for col in range(n):
        pivot = max(range(col, n), key=lambda r: abs(a[r][col]))
        if abs(a[pivot][col]) < 1e-12:
            return None
        a[pivot], a[col] = a[col], a[pivot]
        divisor = a[col][col]
        a[col] = [x / divisor for x in a[col]]
        for row in range(n):
            if row != col:
                scale = a[row][col]
                a[row] = [x - scale * y for x, y in zip(a[row], a[col])]
    return [row[-1] for row in a]


def nash(matrix, tolerance=1e-6, max_iterations=4000):
    n = len(matrix)
    if not n or any(len(row) != n or any(v is None or not math.isfinite(v) for v in row) for row in matrix):
        raise ValueError("Nash needs a complete finite square payoff matrix; missing matchups are not zero")
    if tolerance <= 0 or max_iterations < 1:
        raise ValueError("invalid convergence settings")
    # Iterated strict pure-strategy dominance. Return exactly zero for these rows.
    active = list(range(n))
    while True:
        removed = [i for i in active if any(
            j != i and all(matrix[j][k] > matrix[i][k] + tolerance for k in active) for j in active)]
        if not removed:
            break
        active = [i for i in active if i not in removed]
    weights = [1 / len(active) if i in active else 0.0 for i in range(n)]
    best, best_gap = weights[:], regret(matrix, weights)
    scale = max(1.0, max(max(row) for row in matrix) - min(min(row) for row in matrix))
    method = "replicator"
    iteration = 0
    for iteration in range(max_iterations):
        gap = regret(matrix, weights)
        if gap < best_gap:
            best, best_gap = weights[:], gap
        if gap <= tolerance:
            best, best_gap = weights, gap
            break
        utility = [sum(a * p for a, p in zip(row, weights)) for row in matrix]
        peak = max(utility)
        weights = [p * math.exp((u - peak) / scale) for p, u in zip(weights, utility)]
        total = sum(weights)
        weights = [p / total for p in weights]
    if best_gap > tolerance:
        # A symmetric game's column player receives A^T. Solve A_SS p = v1,
        # then verify *every* excluded row's payoff, including dominated rows.
        method = "support-enumeration"
        for size in range(len(active), 0, -1):
            for support in combinations(active, size):
                equations = [[matrix[i][j] for j in support] + [-1.0] for i in support]
                equations.append([1.0] * size + [0.0])
                solution = solve_linear(equations, [0.0] * size + [1.0])
                if solution is None or min(solution[:-1]) < -1e-10:
                    continue
                trial = [0.0] * n
                for i, p in zip(support, solution[:-1]):
                    trial[i] = max(0.0, p)
                total = sum(trial)
                trial = [p / total for p in trial]
                gap = regret(matrix, trial)
                if gap < best_gap:
                    best, best_gap = trial, gap
                if best_gap <= tolerance:
                    break
            if best_gap <= tolerance:
                break
    # Only cosmetic dust is removed; certify again after changing weights.
    clean = [0.0 if p < 1e-12 else p for p in best]
    clean = [p / sum(clean) for p in clean]
    if regret(matrix, clean) <= best_gap + 1e-12:
        best, best_gap = clean, regret(matrix, clean)
    return {"weights": best, "regret": best_gap, "tolerance": tolerance,
            "converged": best_gap <= tolerance, "method": method,
            "iterations": iteration + 1, "strictly_dominated": [i for i in range(n) if i not in active]}


def diversity(matrix, equilibrium, names):
    weights = equilibrium["weights"]
    entropy = -sum(p * math.log(p) for p in weights if p > 0)
    # Two distinct definitions: an always-best response, and a head-to-head
    # winner against every other opening. Neither is inferred from win share.
    dominant = [names[i] for i in range(len(names)) if all(
        all(matrix[i][k] >= matrix[j][k] - 1e-6 for k in range(len(names)))
        and any(matrix[i][k] > matrix[j][k] + 1e-6 for k in range(len(names)))
        for j in range(len(names)) if j != i)] if len(names) > 1 else []
    beats_all = [names[i] for i in range(len(names)) if all(
        matrix[i][j] > matrix[j][i] + 1e-6 for j in range(len(names)) if i != j)] if len(names) > 1 else []
    return {"entropy": entropy, "normalized_entropy": entropy / math.log(len(names)) if len(names) > 1 else 0.0,
            "openings_ge_10pct": sum(p >= 0.1 - 1e-12 for p in weights),
            "dominant": dominant, "beats_all": beats_all,
            "has_dominant": bool(dominant or beats_all)}
