"""
Vendored reliability coefficients (no SciPy).

References (informal): Gwet AC1 for nominal agreement; Krippendorff alpha
interval metric for ordinal ratings on a fixed 1..k scale.
"""

from __future__ import annotations

import math
from collections import Counter


def gwet_ac1_nominal(xs: list[str], ys: list[str], categories: list[str] | None = None) -> float:
    """Gwet's AC1 for two raters on nominal labels (same length)."""
    if not xs or len(xs) != len(ys):
        return float("nan")
    cats = sorted(set(categories) if categories else set(xs) | set(ys))
    if len(cats) < 2:
        return float("nan")
    n = len(xs)
    po = sum(1 for a, b in zip(xs, ys, strict=True) if a == b) / n
    # pooled category prevalence across both raters
    counts: Counter[str] = Counter()
    for a, b in zip(xs, ys, strict=True):
        counts[a] += 1
        counts[b] += 1
    denom = 2 * n
    pe = sum((counts[c] / denom) ** 2 for c in cats)
    if abs(1.0 - pe) < 1e-12:
        return float("nan")
    return (po - pe) / (1.0 - pe)


def gwet_ac2_linear_ordinal(xs: list[int], ys: list[int], k: int = 4) -> float:
    """
    Gwet's AC2 for two raters with linear weights on integer categories 1..k.

    Uses pooled category prevalence over the 2n rater assignments (Gwet 2014).
    """
    if not xs or len(xs) != len(ys):
        return float("nan")

    def w(i: int, j: int) -> float:
        return 1.0 - abs(i - j) / (k - 1)

    n = len(xs)
    po = sum(w(a, b) for a, b in zip(xs, ys, strict=True)) / n
    counts: Counter[int] = Counter()
    for a, b in zip(xs, ys, strict=True):
        counts[a] += 1
        counts[b] += 1
    tot = 2 * n
    pe = 0.0
    for i in range(1, k + 1):
        for j in range(1, k + 1):
            pi = counts.get(i, 0) / tot
            pj = counts.get(j, 0) / tot
            pe += pi * pj * w(i, j)
    if abs(1.0 - pe) < 1e-12:
        return float("nan")
    return (po - pe) / (1.0 - pe)


def krippendorff_alpha_interval_two_raters(
    xs: list[int], ys: list[int], min_cat: int = 1, max_cat: int = 4
) -> float:
    """
    Krippendorff's alpha with interval metric for exactly two raters and n items.

    Coincidence matrix approach for m=2 coders (simplified from full delta matrix).
    """
    if not xs or len(xs) != len(ys):
        return float("nan")
    k = max_cat - min_cat + 1
    n = len(xs)
    # build value pairs; metric squared difference
    def dist(a: int, b: int) -> float:
        return float((a - b) ** 2)

    # Total disagreement observed vs expected under independence
    do = 0.0
    for a, b in zip(xs, ys, strict=True):
        do += dist(a, b)

    # expected: average squared distance for independent draws from marginal of pooled ratings
    pool: list[int] = []
    for a, b in zip(xs, ys, strict=True):
        pool.append(a)
        pool.append(b)
    tot = len(pool)
    if tot == 0:
        return float("nan")
    exp = 0.0
    for v1 in pool:
        for v2 in pool:
            exp += dist(v1, v2)
    exp /= tot * tot

    do /= n
    if abs(exp) < 1e-14:
        return float("nan")
    return 1.0 - do / exp


def bootstrap_stat(
    xs: list[float | int],
    ys: list[float | int],
    stat_fn,
    reps: int = 4000,
    rng=None,
) -> tuple[float, tuple[float, float]]:
    """Bootstrap 95% CI for stat_fn(xs, ys) treating pairs as units."""
    import random

    rng = rng or random.Random(42)
    n = len(xs)
    if n < 2:
        return (float("nan"), (float("nan"), float("nan")))
    base = stat_fn(xs, ys)
    stats: list[float] = []
    for _ in range(reps):
        idx = [rng.randrange(n) for _ in range(n)]
        sx = [xs[i] for i in idx]
        sy = [ys[i] for i in idx]
        v = stat_fn(sx, sy)
        if math.isfinite(v):
            stats.append(v)
    if len(stats) < 50:
        return (base, (float("nan"), float("nan")))
    stats.sort()
    lo = stats[int(0.025 * (len(stats) - 1))]
    hi = stats[int(0.975 * (len(stats) - 1))]
    return (base, (lo, hi))
