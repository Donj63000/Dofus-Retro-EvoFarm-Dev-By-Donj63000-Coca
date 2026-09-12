"""Je vérifie les archives Unix sans compiler le logiciel ni nécessiter macOS."""

from __future__ import annotations

import hashlib
import importlib.util
import os
from pathlib import Path
import plistlib
import shlex
import shutil
import stat
import subprocess
import tarfile
import tempfile
import time
import unittest
from unittest.mock import patch
import zipfile


SPEC = importlib.util.spec_from_file_location("unix_packaging", Path(__file__).with_name("build-unix-release.py"))
assert SPEC and SPEC.loader
packaging = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(packaging)


def desktop_arguments(value: str) -> list[str]:
    # Je reproduis les deux niveaux documentés du format pour vérifier les chemins réellement transmis.
    unescaped = value.replace("\\\\", "\\")
    lexer = shlex.shlex(unescaped, posix=True)
    lexer.whitespace_split = True
    lexer.commenters = ""
    return [argument.replace("\\$", "$").replace("\\`", "`").replace("%%", "%") for argument in lexer]


class UnixPackagingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="evofarm packaging [test] ")
        self.addCleanup(self.temporary.cleanup)
        self.project = Path(self.temporary.name)
        (self.project / "Cargo.toml").write_text('[package]\nname="evofarm"\nversion="0.3.0"\n', encoding="utf-8")
        (self.project / "README.txt").write_text("Dofus Retro EvoFarm [Dev By Donj63000(Coca)]\n", encoding="utf-8")
        (self.project / "logo.png").write_bytes(b"provided logo")
        (self.project / "fond.png").write_bytes(b"provided background")
        (self.project / "personal-data.json").write_text('"private"', encoding="ascii")
        self.commands: list[tuple[list[str], dict[str, str]]] = []

    def fake_tools(self, command: list[str], *, cwd: Path, env: dict[str, str]) -> None:
        self.assertEqual(cwd, self.project.resolve())
        self.commands.append((command, dict(env)))
        if command[0] == "cargo":
            target = command[command.index("--target") + 1]
            binary = packaging.target_directory(self.project, env) / target / "release" / "evofarm"
            binary.parent.mkdir(parents=True, exist_ok=True)
            binary.write_bytes(b"compiled " + target.encode("ascii"))
        elif command[0] == "lipo" and "-create" in command:
            Path(command[-1]).write_bytes(b"universal x86_64 arm64")
        elif command[0] == "sips":
            Path(command[-1]).write_bytes(b"resized icon")
        elif command[0] == "iconutil":
            Path(command[-1]).write_bytes(b"icns data")
        elif command[0] == "codesign" and "--sign" in command:
            signature = Path(command[-1]) / "Contents" / "_CodeSignature" / "CodeResources"
            signature.parent.mkdir()
            signature.write_bytes(b"adhoc resource seal")

    def build(self, platform: str, **kwargs: object) -> tuple[Path, Path]:
        with patch.object(packaging, "run_command", side_effect=self.fake_tools):
            return packaging.build_release(self.project, platform, environment={}, **kwargs)

    def assert_checksum(self, archive: Path, checksum: Path) -> None:
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        self.assertEqual(checksum.read_text("ascii"), f"{digest}  {archive.name}\n")

    def test_linux_archive_contains_only_portable_products_and_executable_modes(self) -> None:
        archive, checksum = self.build("linux")
        self.assertEqual(archive.name, "EvoFarm-Linux-x64.tar.gz")
        self.assertEqual(checksum.name, "SHA256SUMS-linux")
        with tarfile.open(archive) as output:
            self.assertEqual(set(output.getnames()), {
                "EvoFarm", "EvoFarm/evofarm", "EvoFarm/README.txt", "EvoFarm/logo.png",
                "EvoFarm/fr.donj63000.evofarm.desktop", "EvoFarm/install-desktop.sh",
            })
            self.assertEqual(output.extractfile("EvoFarm/README.txt").read(), (self.project / "README.txt").read_bytes())
            self.assertEqual(output.extractfile("EvoFarm/logo.png").read(), (self.project / "logo.png").read_bytes())
            for member in output.getmembers():
                self.assertEqual((member.uid, member.gid, member.uname, member.gname), (0, 0, "", ""))
                self.assertFalse(member.issym() or member.islnk())
                self.assertEqual(member.mode, 0o644 if member.name.endswith((".txt", ".png")) else 0o755)
        self.assert_checksum(archive, checksum)

    def test_linux_cargo_uses_locked_explicit_target_and_project_manifest(self) -> None:
        self.build("linux")
        self.assertEqual(self.commands[0][0], ["cargo", "build", "--manifest-path", str(self.project / "Cargo.toml"),
                                              "--locked", "--release", "--target", "x86_64-unknown-linux-gnu"])

    def test_macos_bundle_contains_both_architectures_icon_signature_and_version(self) -> None:
        archive, checksum = self.build("macos")
        self.assertEqual(archive.name, "EvoFarm-macOS-universal.zip")
        self.assertEqual(checksum.name, "SHA256SUMS-macos")
        with zipfile.ZipFile(archive) as output:
            files = {info.filename for info in output.infolist() if not info.is_dir()}
            self.assertEqual(files, {
                "README.txt", "EvoFarm.app/Contents/Info.plist", "EvoFarm.app/Contents/PkgInfo",
                "EvoFarm.app/Contents/MacOS/evofarm", "EvoFarm.app/Contents/Resources/EvoFarm.icns",
                "EvoFarm.app/Contents/_CodeSignature/CodeResources",
            })
            metadata = plistlib.loads(output.read("EvoFarm.app/Contents/Info.plist"))
            self.assertEqual(metadata["CFBundleIdentifier"], "fr.donj63000.evofarm")
            self.assertEqual(metadata["CFBundleVersion"], "0.3.0")
            self.assertEqual(metadata["CFBundleShortVersionString"], "0.3.0")
            self.assertEqual(metadata["CFBundleExecutable"], "evofarm")
            self.assertEqual(metadata["CFBundleIconFile"], "EvoFarm.icns")
            self.assertEqual(metadata["LSMinimumSystemVersion"], "11.0")
            self.assertTrue(metadata["NSHighResolutionCapable"])
            self.assertEqual(metadata["NSHumanReadableCopyright"], "Dev By Donj63000(Coca)")
            self.assertEqual(output.read("EvoFarm.app/Contents/MacOS/evofarm"), b"universal x86_64 arm64")
            for entry in output.infolist():
                expected = 0o755 if entry.is_dir() or entry.filename.endswith("/evofarm") else 0o644
                self.assertEqual(stat.S_IMODE(entry.external_attr >> 16), expected)
                self.assertEqual(entry.create_system, 3)
        self.assert_checksum(archive, checksum)
        cargo = [command for command, _ in self.commands if command[0] == "cargo"]
        self.assertEqual([command[-1] for command in cargo], list(packaging.MACOS_TARGETS))
        self.assertIn(["lipo", "-verify_arch", "x86_64", "arm64"], [command[:-1] for command, _ in self.commands])
        sign = [command for command, _ in self.commands if command[0] == "codesign"]
        self.assertIn("--timestamp=none", sign[0])
        self.assertEqual(sign[1][:-1], ["codesign", "--verify", "--deep", "--strict", "--all-architectures"])

    def test_all_ten_retina_and_standard_icon_sizes_are_generated(self) -> None:
        self.build("macos")
        commands = [command for command, _ in self.commands if command[0] == "sips"]
        expected = {f"icon_{size}x{size}{'@2x' if scale == 2 else ''}.png": str(size * scale)
                    for size in (16, 32, 128, 256, 512) for scale in (1, 2)}
        self.assertEqual({Path(command[-1]).name: command[5] for command in commands}, expected)
        self.assertTrue(all(command[5] == command[6] for command in commands))
        self.assertTrue(all(command[-3] == str(self.project / "logo.png") for command in commands))

    def test_custom_macos_minimum_is_used_by_compiler_and_bundle(self) -> None:
        with patch.object(packaging, "run_command", side_effect=self.fake_tools):
            archive, _ = packaging.build_release(self.project, "macos", environment={"MACOSX_DEPLOYMENT_TARGET": "12.3"})
        with zipfile.ZipFile(archive) as output:
            self.assertEqual(plistlib.loads(output.read("EvoFarm.app/Contents/Info.plist"))["LSMinimumSystemVersion"], "12.3")
        self.assertTrue(all(env["MACOSX_DEPLOYMENT_TARGET"] == "12.3" for _, env in self.commands))

    def test_invalid_macos_minimum_does_not_start_compilation(self) -> None:
        for version in ("10.15", "eleven", "11; echo bad"):
            with self.subTest(version=version), patch.object(packaging, "run_command") as runner:
                with self.assertRaises(packaging.BuildError):
                    packaging.build_release(self.project, "macos", environment={"MACOSX_DEPLOYMENT_TARGET": version})
                runner.assert_not_called()

    def test_relative_and_absolute_cargo_target_directories(self) -> None:
        for directory in ("custom targets [release]", str(self.project / "absolute target")):
            with self.subTest(directory=directory), patch.object(packaging, "run_command", side_effect=self.fake_tools):
                archive, _ = packaging.build_release(self.project, "linux", environment={"CARGO_TARGET_DIR": directory})
                with tarfile.open(archive) as output:
                    self.assertEqual(output.extractfile("EvoFarm/evofarm").read(), b"compiled x86_64-unknown-linux-gnu")

    def test_custom_output_directory(self) -> None:
        archive, checksum = self.build("linux", output_dir=self.project / "other [output]")
        self.assertEqual(archive.parent, self.project / "other [output]")
        self.assertEqual(checksum.parent, archive.parent)

    def test_optional_licenses_are_preserved_for_both_platforms(self) -> None:
        (self.project / "LICENSE.md").write_text("Author's license\n", encoding="utf-8", newline="\n")
        linux, _ = self.build("linux")
        macos, _ = self.build("macos")
        with tarfile.open(linux) as output:
            self.assertEqual(output.extractfile("EvoFarm/LICENSE.md").read(), b"Author's license\n")
        with zipfile.ZipFile(macos) as output:
            self.assertEqual(output.read("LICENSE.md"), b"Author's license\n")

    def test_required_assets_missing_or_empty_fail_before_compilation(self) -> None:
        for filename in ("Cargo.toml", "logo.png", "fond.png", "README.txt"):
            path = self.project / filename
            original = path.read_bytes()
            for empty in (False, True):
                with self.subTest(filename=filename, empty=empty), patch.object(packaging, "run_command") as runner:
                    path.write_bytes(b"") if empty else path.unlink()
                    with self.assertRaises(packaging.BuildError):
                        packaging.build_release(self.project, "linux", environment={})
                    runner.assert_not_called()
                    path.write_bytes(original)

    def test_wrong_project_or_prerelease_version_is_rejected(self) -> None:
        for manifest in ('[package]\nname="other"\nversion="0.3.0"', '[package]\nname="evofarm"\nversion="0.3.0-beta.1"'):
            with self.subTest(manifest=manifest), patch.object(packaging, "run_command") as runner:
                (self.project / "Cargo.toml").write_text(manifest, encoding="ascii")
                with self.assertRaises(packaging.BuildError):
                    packaging.build_release(self.project, "linux", environment={})
                runner.assert_not_called()

    def test_missing_or_empty_compiled_binary_is_rejected(self) -> None:
        for empty in (False, True):
            def compile_without_product(command: list[str], *, cwd: Path, env: dict[str, str]) -> None:
                if empty:
                    path = self.project / "target" / packaging.LINUX_TARGET / "release" / "evofarm"
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.touch()
            with self.subTest(empty=empty), patch.object(packaging, "run_command", side_effect=compile_without_product):
                with self.assertRaises(packaging.BuildError):
                    packaging.build_release(self.project, "linux", environment={})

    def test_native_tool_failures_preserve_previous_distribution(self) -> None:
        destination = self.project / "dist"
        destination.mkdir()
        archive = destination / packaging.ARCHIVES["macos"]
        checksum = destination / "SHA256SUMS-macos"
        for failing_tool in ("cargo", "lipo", "sips", "iconutil", "codesign"):
            archive.write_bytes(b"previous complete archive")
            checksum.write_bytes(b"previous checksum")
            def failing(command: list[str], *, cwd: Path, env: dict[str, str]) -> None:
                if command[0] == failing_tool:
                    raise packaging.BuildError("injected tool failure")
                self.fake_tools(command, cwd=cwd, env=env)
            with self.subTest(tool=failing_tool), patch.object(packaging, "run_command", side_effect=failing):
                with self.assertRaises(packaging.BuildError):
                    packaging.build_release(self.project, "macos", environment={})
                self.assertEqual(archive.read_bytes(), b"previous complete archive")
                self.assertEqual(checksum.read_bytes(), b"previous checksum")
                self.assertEqual(sorted(path.name for path in destination.iterdir()), sorted([archive.name, checksum.name]))

    def test_failed_signature_verification_never_publishes_archive(self) -> None:
        def failing(command: list[str], *, cwd: Path, env: dict[str, str]) -> None:
            if command[:2] == ["codesign", "--verify"]:
                raise packaging.BuildError("signature invalid")
            self.fake_tools(command, cwd=cwd, env=env)
        with patch.object(packaging, "run_command", side_effect=failing):
            with self.assertRaises(packaging.BuildError):
                packaging.build_release(self.project, "macos", environment={})
        self.assertEqual(list((self.project / "dist").iterdir()), [])

    def test_archive_failure_preserves_previous_archive_and_checksum(self) -> None:
        archive, checksum = self.build("linux")
        previous = (archive.read_bytes(), checksum.read_bytes())
        with patch.object(packaging, "create_linux_archive", side_effect=OSError("disk full")):
            with self.assertRaises(OSError):
                self.build("linux")
        self.assertEqual((archive.read_bytes(), checksum.read_bytes()), previous)

    def test_command_runner_reports_missing_tool_and_nonzero_exit(self) -> None:
        for error in (FileNotFoundError("cargo"), subprocess.CalledProcessError(2, ["cargo", "build"])):
            with self.subTest(error=error), patch.object(packaging.subprocess, "run", side_effect=error):
                with self.assertRaisesRegex(packaging.BuildError, "cargo"):
                    packaging.run_command(["cargo", "build"], cwd=self.project, env={})

    def test_unsupported_platform_is_rejected(self) -> None:
        with self.assertRaises(packaging.BuildError):
            self.build("android")

    def test_portable_desktop_command_passes_location_as_data(self) -> None:
        entry = packaging.desktop_entry()
        command = next(line[5:] for line in entry.splitlines() if line.startswith("Exec="))
        self.assertEqual(desktop_arguments(command), ["sh", "-c", 'test -n "$1" && exec "$(dirname -- "$1")/evofarm"', "evofarm", "%k"])
        self.assertIn("Icon=fr.donj63000.evofarm\n", entry)

    @unittest.skipUnless(os.name == "posix" and shutil.which("sh"), "Je vérifie le lanceur avec un shell Unix natif.")
    def test_linux_optional_installation_and_launch_with_special_path_characters(self) -> None:
        directory = self.project / 'portable espace [test] "citation" $argent `accent` %k \\ barre'
        directory.mkdir()
        marker = self.project / "launch-confirmed"
        (directory / "evofarm").write_text('#!/bin/sh\nprintf "%s" "$0" > "$EVOTEST_MARKER"\n', encoding="utf-8")
        (directory / "evofarm").chmod(0o755)
        (directory / "logo.png").write_bytes(b"logo")
        installer = directory / "install-desktop.sh"
        installer.write_text(packaging.desktop_installer(), encoding="utf-8")
        env = dict(os.environ, XDG_DATA_HOME=str(self.project / "user data"), EVOTEST_MARKER=str(marker))
        subprocess.run(["sh", str(installer)], check=True, env=env, capture_output=True)
        installed = Path(env["XDG_DATA_HOME"]) / "applications" / "fr.donj63000.evofarm.desktop"
        command = next(line[5:] for line in installed.read_text("utf-8").splitlines() if line.startswith("Exec="))
        arguments = desktop_arguments(command)
        self.assertEqual(arguments, ["sh", "-c", 'exec "$1"', "evofarm", str(directory / "evofarm")])
        subprocess.run(arguments, check=True, env=env)
        self.assertEqual(marker.read_text("utf-8"), str(directory / "evofarm"))
        icon = Path(env["XDG_DATA_HOME"]) / "icons/hicolor/512x512/apps/fr.donj63000.evofarm.png"
        self.assertEqual(icon.read_bytes(), b"logo")
        portable = directory / "fr.donj63000.evofarm.desktop"
        portable.write_text(packaging.desktop_entry(), encoding="utf-8")
        command = next(line[5:] for line in portable.read_text("utf-8").splitlines() if line.startswith("Exec="))
        arguments = desktop_arguments(command)
        arguments[-1] = str(portable)
        subprocess.run(arguments, check=True, env=env)
        self.assertEqual(marker.read_text("utf-8"), str(directory / "evofarm"))
        if shutil.which("gio"):
            # Je vérifie aussi l'interprétation réelle du fichier Desktop Entry par GLib.
            for launcher in (installed,):
                marker.unlink()
                launched = subprocess.run(["gio", "launch", str(launcher)], env=env, capture_output=True)
                self.assertEqual(launched.returncode, 0, launched.stderr.decode("utf-8", errors="replace") + launcher.read_text("utf-8"))
                deadline = time.monotonic() + 5
                while not marker.exists() and time.monotonic() < deadline:
                    time.sleep(0.02)
                self.assertTrue(marker.exists(), launched.stderr.decode("utf-8", errors="replace") + launcher.read_text("utf-8"))
                self.assertEqual(marker.read_text("utf-8"), str(directory / "evofarm"))

    @unittest.skipUnless(os.name == "posix" and shutil.which("sh"), "Je vérifie le script avec un shell Unix natif.")
    def test_portable_desktop_refuses_an_unknown_launcher_location(self) -> None:
        executable = self.project / "evofarm"
        executable.write_text('#!/bin/sh\ntouch launched\n', encoding="utf-8")
        executable.chmod(0o755)
        command = next(line[5:] for line in packaging.desktop_entry().splitlines() if line.startswith("Exec="))
        arguments = desktop_arguments(command)
        arguments[-1] = ""
        result = subprocess.run(arguments, cwd=self.project, capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.project / "launched").exists())

    @unittest.skipUnless(os.name == "posix" and shutil.which("sh"), "Je vérifie le script avec un shell Unix natif.")
    def test_installer_rejects_relative_xdg_data_home(self) -> None:
        installer = self.project / "install-desktop.sh"
        installer.write_text(packaging.desktop_installer(), encoding="utf-8")
        result = subprocess.run(["sh", str(installer)], env=dict(os.environ, XDG_DATA_HOME="relative/path"), capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.project / "relative").exists())


if __name__ == "__main__":
    unittest.main()
