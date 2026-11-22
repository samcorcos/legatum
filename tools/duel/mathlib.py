"""Statistics use independent maps, never treat mirrored seats as independent."""
import itertools
import math
import statistics


def interval(values, alpha=0.05):
    """Conservative, finite-sample Hoeffding CI for a mean of map scores in [0,1]."""
    if not values:
        return [None, None]
    if any(not 0 <= x <= 1 for x in values):
        raise ValueError('scores must be in [0,1]')
    mean = statistics.mean(values)
    radius = math.sqrt(math.log(2 / alpha) / (2 * len(values)))
    return [max(0.0, mean - radius), min(1.0, mean + radius)]


def sign_p(positive, negative):
    """Exact two-sided sign test; ties carry no evidence about direction."""
    n = positive + negative
    if not n:
        return 1.0
    return min(1.0, 2 * sum(math.comb(n, k) for k in range(min(positive, negative) + 1)) / 2**n)


def direction_p(values):
    # With binary game outcomes, each map is a sweep or a tied pair. Under
    # mean=.5, the two sweep probabilities are equal: the sign test is exact.
    if all(value in (0, .5, 1) for value in values):
        return sign_p(sum(v > .5 for v in values), sum(v < .5 for v in values))
    # A completed draw can produce .25/.75 map scores. A median test would no
    # longer test the mean; use a bounded-mean tail bound in that rare case.
    delta = abs(statistics.mean(values) - .5)
    return min(1.0, 2 * math.exp(-2 * len(values) * delta**2))


def holm(p_values):
    """Family-wise adjusted p-values, preserving input order."""
    result = [1.0] * len(p_values)
    previous = 0.0
    for rank, index in enumerate(sorted(range(len(p_values)), key=p_values.__getitem__)):
        previous = max(previous, min(1.0, (len(p_values) - rank) * p_values[index]))
        result[index] = previous
    return result


def cycles(names, cells):
    """Directed triangles whose three edges have Holm-adjusted p < .05."""
    result = []
    for a, b, c in itertools.combinations(names, 3):
        for triangle in [(a, b, c), (a, c, b)]:
            edges = list(zip(triangle, triangle[1:] + triangle[:1]))
            if all(cells[x][y]['significant'] for x, y in edges):
                result.append(list(triangle) + [triangle[0]])
    return result
