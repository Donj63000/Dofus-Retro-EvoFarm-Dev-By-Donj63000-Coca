"""Oracles numériques indépendants et tests réels du lanceur de validation.

Ces tests Python NE compilent et N'EXÉCUTENT PAS le code Rust. Les oracles
vérifient les propriétés du calcul proposé ; les équivalents natifs résident
dans src/security_tests et doivent aussi passer avec cargo test.
"""
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import math
import random
import struct
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("validate_security", ROOT / "scripts/validate-security.py")
assert spec is not None and spec.loader is not None
validator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)


def f32(value: float) -> float:
    return struct.unpack("<f", struct.pack("<f", value))[0]


def aggregate(sessions: list[tuple[int, int, float, int]], buckets: int = 256):
    """Même répartition par différences, indépendante des structures Rust/UI."""
    origin = min(start for start, _, _, _ in sessions)
    finish = max(end for _, end, _, _ in sessions)
    span = finish - origin
    width = (span + buckets - 1) // buckets
    count = (span + width - 1) // width
    partial = [[0.0] * count for _ in range(8)]
    differences = [[0.0] * (count + 1) for _ in range(8)]
    counts = [[0] * (count + 1) for _ in range(8)]
    for start, end, value, category in sessions:
        start -= origin
        end -= origin
        channel = category * 2 + int(value < 0)
        first, last = start // width, (end - 1) // width
        counts[channel][first] += 1
        counts[channel][last + 1] -= 1
        if first == last:
            partial[channel][first] += value
        else:
            rate = value / (end - start)
            partial[channel][first] += rate * ((first + 1) * width - start)
            partial[channel][last] += rate * (end - last * width)
            differences[channel][first + 1] += rate * width
            differences[channel][last] -= rate * width
    result = []
    for channel in range(8):
        running, active = 0.0, 0
        for bucket in range(count):
            running += differences[channel][bucket]
            active += counts[channel][bucket]
            value = (partial[channel][bucket] + running) if active else 0.0
            result.append((origin + bucket * width, min(origin + (bucket + 1) * width, finish), value, channel))
    return result


class AggregationOracleTests(unittest.TestCase):
    def compare_reference(self, sessions):
        bars = aggregate(sessions)
        self.assertLessEqual(len(bars), 8 * 256)
        for start, end, actual, channel in bars:
            expected = math.fsum(value * max(0, min(end, finish) - max(start, begin)) / (finish - begin)
                                 for begin, finish, value, category in sessions
                                 if category * 2 + int(value < 0) == channel)
            self.assertTrue(math.isclose(actual, expected, rel_tol=1e-9, abs_tol=1e-7), (actual, expected))
        self.assertAlmostEqual(math.fsum(bar[2] for bar in bars), math.fsum(session[2] for session in sessions), places=5)

    def test_random_intervals_against_naive_reference(self):
        generator = random.Random(20260913)
        for _ in range(12):
            sessions = []
            for index in range(150):
                start = generator.randrange(10000)
                end = start + generator.randrange(1, 89999)
                value = generator.randrange(-100000, 100001)
                sessions.append((start, end, float(value), index % 4))
            self.compare_reference(sessions)

    def test_touching_intervals_are_half_open(self):
        self.compare_reference([(0, 60, 60.0, 0), (60, 120, 120.0, 0)])

    def test_one_second_sessions_and_both_signs(self):
        self.compare_reference([(index, index + 1, (-1.0 if index % 2 else 1.0) * 1234, index % 4) for index in range(129)])

    def test_sparse_sessions_over_thousands_of_years(self):
        self.compare_reference([(0, 60, 1234.0, 0), (253000000000, 253000000001, -123.0, 1)])

    def test_ten_thousand_overlapping_sessions_remain_bounded(self):
        bars = aggregate([(index % 100, index % 100 + 3600, 100.0, index % 4) for index in range(10000)])
        self.assertLessEqual(len(bars), 2048)
        self.assertTrue(math.isclose(math.fsum(bar[2] for bar in bars), 1000000.0, rel_tol=1e-10))

    def test_negative_and_positive_values_never_share_a_channel(self):
        bars = aggregate([(0, 60, 1000.0, 0), (0, 60, -1000.0, 0)])
        self.assertGreater(math.fsum(value for _, _, value, channel in bars if channel == 0), 0)
        self.assertLess(math.fsum(value for _, _, value, channel in bars if channel == 1), 0)


class NumericBudgetTests(unittest.TestCase):
    def test_maximum_arena_rate_and_total_do_not_overflow_f32(self):
        revenue = f32(f32(1e12) * 100000)
        minimum_minutes = f32(1.0 / 60.0)
        rate = f32(f32(revenue / minimum_minutes) * 60.0)
        self.assertTrue(math.isfinite(rate))
        self.assertLess(rate, 1e21)
        self.assertTrue(math.isfinite(f32(rate * 10000)))

    def test_duration_limit_is_below_integer_and_chrono_seconds_limits(self):
        self.assertEqual(24 * 3600 + 59 * 60 + 59, 89999)
        self.assertLess(89999, 2**32)
        self.assertGreater((2**32 - 1) * 3600, 2**32 - 1)

    def test_compact_json_can_cross_eight_mib_after_pretty_serialization(self):
        entry = {"name": "A" * 512, "character_class": None, "recorded_at": None,
                 "round_time_minutes": 1.0, "seat_price": 1.0, "seats_sold": 1,
                 "capture_price": 1.0, "captures_count": 1, "gross_revenue": 1.0,
                 "total_capture_cost": 1.0, "net_profit": 0.0, "kamas_per_hour": 0.0}
        data = {"schema_version": 2, "data": {"zones": [], "dungeons": [], "duo_trios": [], "arenas": [entry] * 10000}, "drafts": {}}
        compact = json.dumps(data, separators=(",", ":"))
        pretty = json.dumps(data, indent=2)
        self.assertLess(len(compact), 8 * 1024 * 1024)
        self.assertGreater(len(pretty), 8 * 1024 * 1024)

    def test_direct_libc_dependency_reuses_one_locked_registry_package(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        self.assertEqual(manifest["target"]["cfg(unix)"]["dependencies"]["libc"], "0.2")
        libc = [package for package in lock["package"] if package["name"] == "libc"]
        self.assertEqual(len(libc), 1)
        self.assertEqual(len(libc[0]["checksum"]), 64)
        app = next(package for package in lock["package"] if package["name"] == "evofarm")
        self.assertIn("libc", app["dependencies"])


class NativeValidationRunnerTests(unittest.TestCase):
    def invoke(self, cargo, returncodes, apply_format=False):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            argv = ["validate-security.py"] + (["--format"] if apply_format else [])
            with mock.patch.object(validator, "ROOT", root), mock.patch.object(validator.shutil, "which", return_value=cargo), \
                 mock.patch.object(validator.subprocess, "run", side_effect=[mock.Mock(returncode=code) for code in returncodes]) as run, \
                 mock.patch.object(validator.sys, "argv", argv), contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                code = validator.main()
                report = json.loads((root / "target/security-validation/result.json").read_text())
                return code, report, run.call_args_list

    def test_absent_rust_is_not_reported_as_a_pass(self):
        code, report, calls = self.invoke(None, [])
        self.assertEqual(code, 1)
        self.assertFalse(report["passed"])
        self.assertEqual(report["results"][0]["status"], "NOT_RUN")
        self.assertEqual(calls, [])

    def test_format_failure_prevents_a_false_global_pass(self):
        code, report, calls = self.invoke("cargo", [1])
        self.assertEqual(code, 1)
        self.assertFalse(report["complete"])
        self.assertEqual(len(calls), 1)

    def test_audit_failure_blocks_publication(self):
        code, report, _ = self.invoke("cargo", [0, 0, 0, 0, 1])
        self.assertEqual(code, 1)
        self.assertTrue(report["complete"])
        self.assertFalse(report["passed"])
        self.assertEqual(report["results"][-1]["step"], "dependency-audit")

    def test_all_required_steps_must_succeed(self):
        code, report, calls = self.invoke("cargo", [0] * 5)
        self.assertEqual(code, 0)
        self.assertTrue(report["passed"])
        self.assertEqual(len(calls), 5)
        for call in calls:
            self.assertNotIn("shell", call.kwargs)

    def test_source_formatting_requires_explicit_option(self):
        code, report, calls = self.invoke("cargo", [0] * 6, apply_format=True)
        self.assertEqual(code, 0)
        self.assertTrue(report["passed"])
        self.assertEqual(calls[0].args[0], ["cargo", "fmt", "--all"])
        self.assertEqual(calls[1].args[0], ["cargo", "fmt", "--all", "--", "--check"])


if __name__ == "__main__":
    unittest.main()
