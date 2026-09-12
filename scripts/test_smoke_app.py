"""Je teste la détection des arrêts précoces et les dépendances de démarrage."""

import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest
from unittest import mock


spec = importlib.util.spec_from_file_location("smoke_app", Path(__file__).with_name("smoke-app.py"))
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


class StartupTests(unittest.TestCase):
    def test_running_child_is_terminated_and_its_log_kept(self):
        log = smoke.check_startup([sys.executable, "-u", "-c", "import time; print('ready'); time.sleep(30)"], 0.5)
        self.assertIn("ready", log)

    def test_early_crash_keeps_diagnostic(self):
        with self.assertRaisesRegex(RuntimeError, "code 7.*") as error:
            smoke.check_startup([sys.executable, "-c", "print('missing library'); raise SystemExit(7)"])
        self.assertIn("missing library", str(error.exception))

    def test_early_successful_exit_is_not_a_running_application(self):
        with self.assertRaisesRegex(RuntimeError, "code 0"):
            smoke.check_startup([sys.executable, "-c", "pass"])

    def test_missing_binary_fails(self):
        with self.assertRaises(OSError):
            smoke.check_startup([str(Path(__file__).with_name("missing-binary"))])

    def test_macos_system_dependencies_and_two_slices(self):
        libraries = "binary:\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)\n"
        minima = "cmd LC_BUILD_VERSION\ncmdsize 32\nplatform 1\nminos 11.0\n" * 2
        with mock.patch.object(smoke.subprocess, "run", side_effect=[subprocess.CompletedProcess([], 0, libraries), subprocess.CompletedProcess([], 0, minima)]):
            smoke.check_macos_dependencies(Path("binary"))

    def test_macos_rejects_homebrew_dependency(self):
        result = subprocess.CompletedProcess([], 0, "\t/opt/homebrew/lib/example.dylib (compatibility version 1.0.0)\n")
        with mock.patch.object(smoke.subprocess, "run", return_value=result):
            with self.assertRaisesRegex(RuntimeError, "extérieure"):
                smoke.check_macos_dependencies(Path("binary"))

    def test_macos_rejects_newer_os_requirement(self):
        result = subprocess.CompletedProcess([], 0, "cmd LC_BUILD_VERSION\ncmdsize 32\nplatform MACOS\nminos 15.0\n")
        with mock.patch.object(smoke.subprocess, "run", return_value=result):
            with self.assertRaisesRegex(RuntimeError, "au-delà"):
                smoke.check_macos_dependencies(Path("binary"))


if __name__ == "__main__":
    unittest.main()
