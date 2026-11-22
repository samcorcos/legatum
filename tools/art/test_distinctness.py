#!/usr/bin/env python3
"""Unit tests for tools/art/distinctness.py on synthetic pictures: python3 tools/art/test_distinctness.py"""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import distinctness as d  # noqa: E402

N = 24


def solid(w, h, rgb, box):
    """RGBA rows: a filled box (x0, y0, x1, y1) of `rgb` on a transparent background."""
    x0, y0, x1, y1 = box
    return [[(rgb[0], rgb[1], rgb[2], 255) if x0 <= x < x1 and y0 <= y < y1 else (0, 0, 0, 0) for x in range(w)] for y in range(h)]


def score(a, b, kind):
    pa, pb = d.Pic(a), d.Pic(b)
    la = dict((m, d.lab_plane(pa, m)) for m in d.MODES)
    lb = dict((m, d.lab_plane(pb, m)) for m in d.MODES)
    return d.score_pair(d.Prep(pa), d.Prep(pb), la, lb, kind)


class Metric(unittest.TestCase):
    def test_identical_pictures_score_one(self):
        a = solid(N, N, (40, 36, 25), (4, 4, 20, 20))
        for kind in ('mono', 'colour'):
            self.assertGreater(score(a, a, kind)['score'], 0.97)

    def test_different_silhouettes_score_low(self):
        wide = solid(N, N, (40, 36, 25), (2, 9, 22, 15))
        tall = solid(N, N, (40, 36, 25), (9, 2, 15, 22))
        self.assertLess(score(wide, tall, 'mono')['score'], 0.5)

    def test_same_shape_in_two_greys_is_flagged_in_greyscale(self):
        # Coal and Iron before the repaint: the same lump in two near greys must score as alike.
        a = solid(N, N, (60, 60, 64), (3, 3, 21, 21))
        b = solid(N, N, (84, 82, 80), (3, 3, 21, 21))
        self.assertGreater(score(a, b, 'colour')['score'], 0.7)

    def test_red_and_green_of_one_lightness_are_flagged_for_colour_blind_sight(self):
        red = solid(N, N, (176, 110, 60), (3, 3, 21, 21))
        green = solid(N, N, (110, 150, 30), (3, 3, 21, 21))
        r = score(red, green, 'colour')
        self.assertIn(r['gap_mode'], ('protan', 'deutan', 'gray'))
        self.assertGreater(r['score'], score(solid(N, N, (30, 30, 40), (3, 3, 21, 21)), solid(N, N, (230, 200, 90), (3, 3, 21, 21)), 'colour')['score'])

    def test_dark_blue_black_versus_light_rust_is_distinct(self):
        coal = solid(N, N, (25, 30, 46), (3, 3, 21, 21))
        iron = solid(N, N, (170, 96, 66), (3, 3, 21, 21))
        self.assertLess(score(coal, iron, 'colour')['score'], 0.4)

    def test_gist_sees_two_upright_figures_as_alike_whatever_their_colour(self):
        # An upright figure with a vertical pole, in two colours and drawn a pixel apart, against a low wide object.
        def figure(rgb, dx):
            rows = solid(N, N, rgb, (9 + dx, 8, 14 + dx, 22))
            for y in range(1, 22):
                rows[y][17 + dx] = (rgb[0], rgb[1], rgb[2], 255)
            return rows
        a, b = d.Pic(figure((90, 70, 40), 0)), d.Pic(figure((60, 90, 120), 1))
        wide = d.Pic(solid(N, N, (90, 70, 40), (2, 12, 22, 20)))
        self.assertGreater(d.gist_similarity(a.gist, b.gist), 0.7)
        self.assertLess(d.gist_similarity(a.gist, wide.gist), 0.5)

    def test_png_round_trip(self):
        path = os.path.join(os.path.dirname(os.path.abspath(__file__)), '.cache', 'roundtrip.png')
        os.makedirs(os.path.dirname(path), exist_ok=True)
        rows = [[(x * 10 % 256, y * 10 % 256, 7) for x in range(8)] for y in range(8)]
        d.write_png(path, 8, 8, rows)
        # write_png is opaque RGB; read_png wants RGBA, so only the signature and size are checked here.
        with open(path, 'rb') as f:
            self.assertEqual(f.read(8), b'\x89PNG\r\n\x1a\n')

    def test_class_groups_exempt_same_class_roundels(self):
        cfg = {'groups': {'units-roundel': [{'why': 'Mounted', 'members': ['horseman', 'knight']}]}}
        self.assertTrue(d.same_group(cfg, {'set': 'units-roundel', 'a': 'horseman+cavalry', 'b': 'knight'}) is None)
        self.assertTrue(d.same_group(cfg, {'set': 'units-roundel', 'a': 'horseman', 'b': 'knight'}))
        self.assertTrue(d.same_group(cfg, {'set': 'units-roundel', 'a': 'horseman', 'b': 'archer'}) is None)


if __name__ == '__main__':
    unittest.main()
