"""Je teste les refus de publication avec des archives synthétiques et isolées."""

from __future__ import annotations

import importlib.util
import gzip
import io
import json
from pathlib import Path
import plistlib
import stat
import struct
import tarfile
import tempfile
import unittest
import zipfile


SPEC = importlib.util.spec_from_file_location("verify_release", Path(__file__).with_name("verify-release.py"))
assert SPEC is not None and SPEC.loader is not None
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)
VERSION = "0.3.0"


def pe_fixture(machine: int = 0x8664) -> bytes:
    data = bytearray(512)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 0x3C, 128)
    data[128:132] = b"PE\0\0"
    struct.pack_into("<HH", data, 132, machine, 1)
    struct.pack_into("<HHH", data, 148, 240, 0x0002, 0x20B)
    return bytes(data)


def elf_fixture(machine: int = 62) -> bytes:
    data = bytearray(128)
    data[:7] = b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHI", data, 16, 3, machine, 1)
    struct.pack_into("<H", data, 52, 64)
    return bytes(data)


def macho_fixture(fat64: bool = False) -> bytes:
    data = bytearray(8232)
    struct.pack_into(">II", data, 0, 0xCAFEBABF if fat64 else 0xCAFEBABE, 2)
    for index, cpu in enumerate((0x01000007, 0x0100000C)):
        offset = 4096 * (index + 1)
        if fat64:
            struct.pack_into(">IIQQII", data, 8 + index * 32, cpu, 3, offset, 40, 12, 0)
        else:
            struct.pack_into(">IIIII", data, 8 + index * 20, cpu, 3, offset, 40, 12)
        struct.pack_into("<6I", data, offset, 0xFEEDFACF, cpu, 3, 2, 1, 8)
    return bytes(data)


def icns_fixture() -> bytes:
    payload = b"\x89PNG\r\n\x1a\n"
    block = b"ic07" + struct.pack(">I", 8 + len(payload)) + payload
    return b"icns" + struct.pack(">I", 8 + len(block)) + block


def plist_fixture() -> bytes:
    return plistlib.dumps({
        "CFBundleIdentifier": "fr.donj63000.evofarm",
        "CFBundleExecutable": "evofarm",
        "CFBundleName": "EvoFarm",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": VERSION,
        "CFBundleVersion": VERSION,
        "CFBundleIconFile": "EvoFarm.icns",
    })


def write_zip(path: Path, entries: dict[str, tuple[bytes, int]]) -> None:
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, (content, mode) in entries.items():
            member = zipfile.ZipInfo(name)
            # Je conserve les chemins malformés des fixtures même sous Windows.
            member.filename = name
            member.create_system = 3
            member.external_attr = mode << 16
            archive.writestr(member, content)


def write_tar(path: Path, entries: dict[str, tuple[bytes, int, bytes]]) -> None:
    with tarfile.open(path, "w:gz") as archive:
        directory = tarfile.TarInfo("EvoFarm")
        directory.type = tarfile.DIRTYPE
        directory.mode = 0o755
        archive.addfile(directory)
        for name, (content, mode, entry_type) in entries.items():
            member = tarfile.TarInfo(name)
            member.size = len(content)
            member.mode = mode
            member.type = entry_type
            if entry_type in {tarfile.SYMTYPE, tarfile.LNKTYPE}:
                member.linkname = "evofarm"
                member.size = 0
                archive.addfile(member)
            else:
                archive.addfile(member, io.BytesIO(content))


class ReleaseValidationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="evofarm-release-validation-")
        self.addCleanup(self.temporary.cleanup)
        self.distribution = Path(self.temporary.name) / "distribution"
        self.distribution.mkdir()
        self.windows = {
            "EvoFarm.exe": (pe_fixture(), stat.S_IFREG | 0o755),
            "README.txt": (b"Documentation Windows", stat.S_IFREG | 0o644),
        }
        self.linux = {
            "EvoFarm/evofarm": (elf_fixture(), 0o755, tarfile.REGTYPE),
            "EvoFarm/README.txt": (b"Documentation Linux", 0o644, tarfile.REGTYPE),
            "EvoFarm/logo.png": (b"\x89PNG\r\n\x1a\n", 0o644, tarfile.REGTYPE),
            "EvoFarm/fr.donj63000.evofarm.desktop": (b"[Desktop Entry]\nType=Application\n", 0o755, tarfile.REGTYPE),
            "EvoFarm/install-desktop.sh": (b"#!/bin/sh\nexit 0\n", 0o755, tarfile.REGTYPE),
        }
        self.macos = {
            "README.txt": (b"Documentation macOS", stat.S_IFREG | 0o644),
            f"{release.BUNDLE}MacOS/evofarm": (macho_fixture(), stat.S_IFREG | 0o755),
            f"{release.BUNDLE}Info.plist": (plist_fixture(), stat.S_IFREG | 0o644),
            f"{release.BUNDLE}PkgInfo": (b"APPL????", stat.S_IFREG | 0o644),
            f"{release.BUNDLE}Resources/EvoFarm.icns": (icns_fixture(), stat.S_IFREG | 0o644),
            f"{release.BUNDLE}_CodeSignature/CodeResources": (plistlib.dumps({"files": {}}), stat.S_IFREG | 0o644),
        }
        self.rebuild()

    def rebuild(self) -> None:
        (self.distribution / release.ARTIFACTS[0]).write_bytes(pe_fixture())
        write_zip(self.distribution / release.ARTIFACTS[1], self.windows)
        write_tar(self.distribution / release.ARTIFACTS[2], self.linux)
        write_zip(self.distribution / release.ARTIFACTS[3], self.macos)
        self.manifest()

    def manifest(self, assemble: bool = False) -> None:
        groups = {"SHA256SUMS": release.ARTIFACTS}
        if assemble:
            groups = {"SHA256SUMS": release.ARTIFACTS[:2],
                      "SHA256SUMS-linux": (release.ARTIFACTS[2],),
                      "SHA256SUMS-macos": (release.ARTIFACTS[3],)}
        for manifest, names in groups.items():
            content = "".join(f"{release.digest_file(self.distribution / name)}  {name}\n" for name in names)
            (self.distribution / manifest).write_text(content, encoding="ascii")

    def verify(self, **kwargs: object) -> None:
        release.verify_release(self.distribution, VERSION, **kwargs)

    def rejected(self, message: str, **kwargs: object) -> None:
        with self.assertRaisesRegex(release.ValidationError, message):
            self.verify(**kwargs)

    def github_fixture(self) -> Path:
        payload = {"tag_name": f"v{VERSION}", "draft": False, "prerelease": False, "assets": [
            {"name": name, "size": (self.distribution / name).stat().st_size,
             "digest": f"sha256:{release.digest_file(self.distribution / name)}"}
            for name in (*release.ARTIFACTS, "SHA256SUMS")
        ]}
        path = Path(self.temporary.name) / "release.json"
        path.write_text(json.dumps(payload), encoding="utf-8")
        return path

    def test_valid_four_platform_artifacts(self) -> None:
        self.verify()

    def test_assemble_validates_then_writes_four_hashes(self) -> None:
        self.manifest(assemble=True)
        originals = {name: (self.distribution / name).read_bytes() for name in release.INTERMEDIATE_MANIFESTS}
        self.verify(assemble=True)
        self.assertEqual(set(release.parse_manifest(self.distribution / "SHA256SUMS", set(release.ARTIFACTS))), set(release.ARTIFACTS))
        self.verify()
        for name, original in originals.items():
            self.assertEqual((self.distribution / name).read_bytes(), original)

    def test_failed_assembly_preserves_windows_manifest(self) -> None:
        self.linux["EvoFarm/evofarm"] = (elf_fixture(), 0o644, tarfile.REGTYPE)
        self.rebuild()
        self.manifest(assemble=True)
        original = (self.distribution / "SHA256SUMS").read_bytes()
        self.rejected("exécution", assemble=True)
        self.assertEqual((self.distribution / "SHA256SUMS").read_bytes(), original)
        self.assertFalse(list(self.distribution.glob(".SHA256SUMS-*")))

    def test_assembly_requires_each_platform_manifest(self) -> None:
        self.manifest(assemble=True)
        (self.distribution / "SHA256SUMS-linux").unlink()
        self.rejected("Manifest absent", assemble=True)

    def test_assembly_refuses_overlapping_manifest_entries(self) -> None:
        self.manifest(assemble=True)
        path = self.distribution / "SHA256SUMS-linux"
        path.write_text(f"{'a' * 64}  EvoFarm.exe\n", encoding="ascii")
        self.rejected("inattendus", assemble=True)

    def test_checksum_mismatch(self) -> None:
        with (self.distribution / "EvoFarm.exe").open("ab") as stream:
            stream.write(b"changed")
        self.rejected("SHA-256 incorrecte")

    def test_duplicate_checksum_entry(self) -> None:
        path = self.distribution / "SHA256SUMS"
        path.write_text(path.read_text() + path.read_text().splitlines()[0] + "\n", encoding="ascii")
        self.rejected("en double")

    def test_missing_checksum_entry(self) -> None:
        path = self.distribution / "SHA256SUMS"
        path.write_text("\n".join(path.read_text().splitlines()[:-1]) + "\n", encoding="ascii")
        self.rejected("manquants")

    def test_invalid_checksum_syntax(self) -> None:
        for line in (f"{'a' * 64} *EvoFarm.exe", f"{'z' * 64}  EvoFarm.exe", f"{'a' * 64}  ../EvoFarm.exe", "", "comment"):
            with self.subTest(line=line):
                (self.distribution / "SHA256SUMS").write_text(line + "\n", encoding="ascii")
                self.rejected("Ligne SHA-256")

    def test_uppercase_checksum_is_accepted(self) -> None:
        path = self.distribution / "SHA256SUMS"
        path.write_text("".join(line[:64].upper() + line[64:] + "\n" for line in path.read_text().splitlines()), encoding="ascii")
        self.verify()

    def test_missing_download(self) -> None:
        (self.distribution / release.ARTIFACTS[2]).unlink()
        self.rejected("Livrable absent")

    def test_empty_download(self) -> None:
        (self.distribution / release.ARTIFACTS[2]).write_bytes(b"")
        self.rejected("Livrable absent")

    def test_unexpected_distribution_file(self) -> None:
        (self.distribution / "debug.log").write_bytes(b"private")
        self.rejected("inattendu")

    def test_unexpected_distribution_directory(self) -> None:
        (self.distribution / "source").mkdir()
        self.rejected("inattendu")

    def test_invalid_release_version(self) -> None:
        for version in ("v0.3.0", "0.3", "../0.3.0", "0.3.0\n"):
            with self.subTest(version=version), self.assertRaisesRegex(release.ValidationError, "Version"):
                release.verify_release(self.distribution, version)

    def test_windows_binary_must_match_zip(self) -> None:
        self.windows["EvoFarm.exe"] = (pe_fixture() + b"different", stat.S_IFREG | 0o755)
        self.rebuild()
        self.rejected("diffère")

    def test_windows_architecture(self) -> None:
        (self.distribution / "EvoFarm.exe").write_bytes(pe_fixture(0x14C))
        self.manifest()
        self.rejected("x86-64")

    def test_windows_rejects_library(self) -> None:
        data = bytearray(pe_fixture())
        struct.pack_into("<H", data, 150, 0x2002)
        with self.assertRaisesRegex(release.ValidationError, "application"):
            release.check_pe(data)

    def test_windows_header_bounds(self) -> None:
        data = bytearray(pe_fixture())
        struct.pack_into("<I", data, 0x3C, 0xFFFFFFFF)
        with self.assertRaisesRegex(release.ValidationError, "tronqué"):
            release.check_pe(data)

    def test_windows_zip_missing_documentation(self) -> None:
        del self.windows["README.txt"]
        self.rebuild()
        self.rejected("requis absents")

    def test_windows_zip_rejects_source_or_private_data(self) -> None:
        for name in ("data.json", "src/main.rs", ".git/config"):
            with self.subTest(name=name):
                self.windows[name] = (b"private", stat.S_IFREG | 0o644)
                self.rebuild()
                self.rejected("inattendu")
                del self.windows[name]

    def test_zip_rejects_traversal_absolute_and_windows_paths(self) -> None:
        for name in ("../data.json", "/etc/passwd", "C:/data", "EvoFarm.app/../data", "foo\\bar", "./README.txt", "foo//bar"):
            with self.subTest(name=name):
                self.windows[name] = (b"unsafe", stat.S_IFREG | 0o644)
                self.rebuild()
                self.rejected("[Cc]hemin")
                del self.windows[name]

    def test_zip_rejects_symbolic_link(self) -> None:
        self.windows["README.txt"] = (b"outside", stat.S_IFLNK | 0o777)
        self.rebuild()
        self.rejected("Lien")

    def test_zip_crc_is_read(self) -> None:
        path = self.distribution / release.ARTIFACTS[1]
        data = path.read_bytes().replace(b"Documentation Windows", b"Documentation changed")
        path.write_bytes(data)
        self.manifest()
        with self.assertRaises(zipfile.BadZipFile):
            self.verify()

    def test_optional_licenses_on_all_platforms(self) -> None:
        self.windows["LICENSE"] = (b"license", stat.S_IFREG | 0o644)
        self.linux["EvoFarm/LICENSE.txt"] = (b"license", 0o644, tarfile.REGTYPE)
        self.macos["LICENSE.md"] = (b"license", stat.S_IFREG | 0o644)
        self.rebuild()
        self.verify()

    def test_linux_binary_architecture(self) -> None:
        self.linux["EvoFarm/evofarm"] = (elf_fixture(183), 0o755, tarfile.REGTYPE)
        self.rebuild()
        self.rejected("x86-64")

    def test_linux_execute_permissions(self) -> None:
        for name in ("evofarm", "install-desktop.sh", "fr.donj63000.evofarm.desktop"):
            with self.subTest(name=name):
                original = self.linux[f"EvoFarm/{name}"]
                self.linux[f"EvoFarm/{name}"] = (original[0], 0o644, original[2])
                self.rebuild()
                self.rejected("exécution")
                self.linux[f"EvoFarm/{name}"] = original

    def test_linux_rejects_writable_and_setuid_modes(self) -> None:
        for mode in (0o777, 0o4755, 0o2755):
            with self.subTest(mode=mode):
                self.linux["EvoFarm/evofarm"] = (elf_fixture(), mode, tarfile.REGTYPE)
                self.rebuild()
                self.rejected("non sûres")

    def test_linux_rejects_symbolic_and_hard_links(self) -> None:
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
            with self.subTest(kind=kind):
                self.linux["EvoFarm/README.txt"] = (b"", 0o644, kind)
                self.rebuild()
                self.rejected("Lien")

    def test_linux_rejects_traversal(self) -> None:
        self.linux["EvoFarm/../data.json"] = (b"unsafe", 0o644, tarfile.REGTYPE)
        self.rebuild()
        self.rejected("dangereux")

    def test_linux_requires_icon(self) -> None:
        del self.linux["EvoFarm/logo.png"]
        self.rebuild()
        self.rejected("requis absents")

    def test_linux_requires_script_interpreter(self) -> None:
        self.linux["EvoFarm/install-desktop.sh"] = (b"echo hello", 0o755, tarfile.REGTYPE)
        self.rebuild()
        self.rejected("interpréteur")

    def test_linux_gzip_crc_is_read(self) -> None:
        path = self.distribution / release.ARTIFACTS[2]
        data = bytearray(path.read_bytes())
        data[-8] ^= 1
        path.write_bytes(data)
        self.manifest()
        with self.assertRaises(gzip.BadGzipFile):
            self.verify()

    def test_macos_accepts_fat64(self) -> None:
        self.macos[f"{release.BUNDLE}MacOS/evofarm"] = (macho_fixture(True), stat.S_IFREG | 0o755)
        self.rebuild()
        self.verify()

    def test_macos_rejects_thin_binary(self) -> None:
        self.macos[f"{release.BUNDLE}MacOS/evofarm"] = (macho_fixture()[4096:4136], stat.S_IFREG | 0o755)
        self.rebuild()
        self.rejected("universel")

    def test_macos_rejects_duplicate_architecture(self) -> None:
        data = bytearray(macho_fixture())
        struct.pack_into(">I", data, 28, 0x01000007)
        with self.assertRaisesRegex(release.ValidationError, "chacune une fois"):
            release.check_macho(data)

    def test_macos_rejects_out_of_bounds_slice(self) -> None:
        data = bytearray(macho_fixture())
        struct.pack_into(">I", data, 20, len(data))
        with self.assertRaisesRegex(release.ValidationError, "hors du fichier"):
            release.check_macho(data)

    def test_macos_rejects_overlapping_slices(self) -> None:
        data = bytearray(macho_fixture())
        struct.pack_into(">I", data, 36, 4096)
        with self.assertRaisesRegex(release.ValidationError, "superposées"):
            release.check_macho(data)

    def test_macos_rejects_mismatched_slice_cpu(self) -> None:
        data = bytearray(macho_fixture())
        struct.pack_into("<I", data, 4100, 0x0100000C)
        with self.assertRaisesRegex(release.ValidationError, "incohérent"):
            release.check_macho(data)

    def test_macos_requires_execute_permission(self) -> None:
        self.macos[f"{release.BUNDLE}MacOS/evofarm"] = (macho_fixture(), stat.S_IFREG | 0o644)
        self.rebuild()
        self.rejected("Permissions")

    def test_macos_requires_matching_version_and_bundle_identity(self) -> None:
        for key in ("CFBundleVersion", "CFBundleShortVersionString", "CFBundleIdentifier", "CFBundleExecutable"):
            with self.subTest(key=key):
                metadata = plistlib.loads(plist_fixture())
                metadata[key] = "incorrect"
                self.macos[f"{release.BUNDLE}Info.plist"] = (plistlib.dumps(metadata), stat.S_IFREG | 0o644)
                self.rebuild()
                self.rejected(key)

    def test_macos_requires_bundle_icon(self) -> None:
        del self.macos[f"{release.BUNDLE}Resources/EvoFarm.icns"]
        self.rebuild()
        self.rejected("requis absents")

    def test_macos_rejects_malformed_icon(self) -> None:
        self.macos[f"{release.BUNDLE}Resources/EvoFarm.icns"] = (b"not an icon", stat.S_IFREG | 0o644)
        self.rebuild()
        self.rejected("ICNS")

    def test_macos_requires_signature_resources(self) -> None:
        del self.macos[f"{release.BUNDLE}_CodeSignature/CodeResources"]
        self.rebuild()
        self.rejected("requis absents")

    def test_github_release_hashes_match_downloads(self) -> None:
        self.verify(github_json=self.github_fixture())

    def test_github_release_rejects_wrong_digest_size_or_name(self) -> None:
        for key, wrong in (("digest", "sha256:" + "0" * 64), ("digest", None), ("size", 0), ("name", "wrong.exe")):
            with self.subTest(key=key, wrong=wrong):
                path = self.github_fixture()
                metadata = json.loads(path.read_text())
                metadata["assets"][0][key] = wrong
                path.write_text(json.dumps(metadata), encoding="utf-8")
                self.rejected("GitHub", github_json=path)

    def test_github_release_rejects_draft_prerelease_or_wrong_tag(self) -> None:
        for key, wrong in (("draft", True), ("prerelease", True), ("tag_name", "v0.2.0")):
            with self.subTest(key=key):
                path = self.github_fixture()
                metadata = json.loads(path.read_text())
                metadata[key] = wrong
                path.write_text(json.dumps(metadata), encoding="utf-8")
                self.rejected("GitHub", github_json=path)

    def test_github_release_requires_exactly_five_assets(self) -> None:
        path = self.github_fixture()
        metadata = json.loads(path.read_text())
        metadata["assets"].append(dict(metadata["assets"][0]))
        path.write_text(json.dumps(metadata), encoding="utf-8")
        self.rejected("Nombre", github_json=path)

    def test_github_release_rejects_malformed_asset_name(self) -> None:
        path = self.github_fixture()
        metadata = json.loads(path.read_text())
        metadata["assets"][0]["name"] = ["EvoFarm.exe"]
        path.write_text(json.dumps(metadata), encoding="utf-8")
        self.rejected("Nom", github_json=path)


if __name__ == "__main__":
    unittest.main()
