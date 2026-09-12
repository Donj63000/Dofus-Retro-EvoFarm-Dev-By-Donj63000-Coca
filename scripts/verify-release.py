#!/usr/bin/env python3
"""Je vérifie les distributions EvoFarm sans extraire ni exécuter leurs fichiers."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import plistlib
import re
import stat
import struct
import sys
import tarfile
import tempfile
import zipfile


ARTIFACTS = (
    "EvoFarm.exe",
    "EvoFarm-Windows-x64.zip",
    "EvoFarm-Linux-x64.tar.gz",
    "EvoFarm-macOS-universal.zip",
)
INTERMEDIATE_MANIFESTS = ("SHA256SUMS-linux", "SHA256SUMS-macos")
LICENSES = {"LICENSE", "LICENSE.txt", "LICENSE.md"}
MAX_MEMBER_SIZE = 512 * 1024 * 1024
MAX_TOTAL_SIZE = 1024 * 1024 * 1024
MAX_MEMBERS = 32
BUNDLE = "EvoFarm.app/Contents/"


class ValidationError(ValueError):
    """Je distingue un livrable incorrect d'une erreur du script."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValidationError(message)


def digest_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_manifest(path: Path, expected: set[str]) -> dict[str, str]:
    require(path.is_file() and not path.is_symlink(), f"Manifest absent ou symbolique : {path.name}")
    require(path.stat().st_size <= 4096, f"Manifest trop volumineux : {path.name}")
    result: dict[str, str] = {}
    for line in path.read_text(encoding="ascii").splitlines():
        match = re.fullmatch(r"([0-9a-fA-F]{64})  ([A-Za-z0-9_.-]+)", line)
        require(match is not None, f"Ligne SHA-256 invalide dans {path.name}")
        checksum, name = match.groups()
        require(name not in result, f"Empreinte en double : {name}")
        result[name] = checksum.lower()
    require(set(result) == expected, f"Fichiers inattendus ou manquants dans {path.name} : {sorted(result)}")
    return result


def safe_member(name: str) -> str:
    require(bool(name) and "\\" not in name and "\x00" not in name, f"Chemin d'archive invalide : {name!r}")
    clean = name.removesuffix("/")
    path = PurePosixPath(clean)
    require(not path.is_absolute() and bool(clean), f"Chemin absolu ou vide : {name!r}")
    require(all(part not in {"", ".", ".."} and ":" not in part for part in clean.split("/")),
            f"Chemin dangereux dans l'archive : {name!r}")
    require(str(path) == clean, f"Chemin non canonique : {name!r}")
    return clean


def check_pe(data: bytes) -> None:
    require(len(data) >= 64 and data[:2] == b"MZ", "L'exécutable Windows n'est pas un PE")
    offset = struct.unpack_from("<I", data, 0x3C)[0]
    require(64 <= offset <= len(data) - 26, "En-tête PE tronqué")
    require(data[offset:offset + 4] == b"PE\0\0", "Signature PE invalide")
    machine, sections = struct.unpack_from("<HH", data, offset + 4)
    optional_size, characteristics = struct.unpack_from("<HH", data, offset + 20)
    require(machine == 0x8664 and sections > 0, "L'exécutable Windows doit cibler x86-64")
    require(characteristics & 0x0002 != 0 and characteristics & 0x2000 == 0,
            "Le fichier Windows doit être une application exécutable")
    require(optional_size >= 112 and offset + 24 + optional_size <= len(data), "En-tête PE64 incomplet")
    require(struct.unpack_from("<H", data, offset + 24)[0] == 0x20B, "Format PE32+ attendu")


def check_elf(data: bytes) -> None:
    require(len(data) >= 64 and data[:7] == b"\x7fELF\x02\x01\x01", "Le binaire Linux doit être ELF64 little-endian")
    file_type, machine, version = struct.unpack_from("<HHI", data, 16)
    require(machine == 62 and file_type in {2, 3} and version == 1, "Le binaire Linux doit être exécutable x86-64")
    require(struct.unpack_from("<H", data, 52)[0] == 64, "Taille d'en-tête ELF64 invalide")


def check_macho(data: bytes) -> None:
    require(len(data) >= 8, "Binaire macOS tronqué")
    magic, count = struct.unpack_from(">II", data)
    require(magic in {0xCAFEBABE, 0xCAFEBABF} and count == 2,
            "Le binaire macOS doit être universel avec deux architectures")
    entry_size = 20 if magic == 0xCAFEBABE else 32
    header_end = 8 + count * entry_size
    require(len(data) >= header_end, "Table universelle macOS tronquée")
    architectures: set[int] = set()
    spans: list[tuple[int, int]] = []
    for index in range(count):
        entry = 8 + index * entry_size
        cpu, subtype = struct.unpack_from(">II", data, entry)
        if entry_size == 20:
            offset, size, alignment = struct.unpack_from(">III", data, entry + 8)
        else:
            offset, size, alignment, reserved = struct.unpack_from(">QQII", data, entry + 8)
            require(reserved == 0, "Champ réservé Mach-O invalide")
        require(cpu in {0x01000007, 0x0100000C} and cpu not in architectures,
                "Les architectures macOS doivent être x86-64 et arm64, chacune une fois")
        require(alignment <= 30 and offset % (1 << alignment) == 0, "Alignement Mach-O invalide")
        require(offset >= header_end and size >= 32 and offset + size <= len(data), "Tranche Mach-O hors du fichier")
        require(all(offset + size <= start or offset >= end for start, end in spans), "Tranches Mach-O superposées")
        magic64, inner_cpu, inner_subtype, file_type, commands, commands_size = struct.unpack_from("<6I", data, offset)
        require(magic64 == 0xFEEDFACF and inner_cpu == cpu and inner_subtype == subtype and file_type == 2,
                "En-tête Mach-O exécutable incohérent")
        require(commands > 0 and commands_size >= commands * 8 and 32 + commands_size <= size,
                "Commandes Mach-O tronquées")
        architectures.add(cpu)
        spans.append((offset, offset + size))


def check_icns(data: bytes) -> None:
    require(len(data) >= 16 and data[:4] == b"icns", "Icône macOS ICNS absente ou invalide")
    require(struct.unpack_from(">I", data, 4)[0] == len(data), "Taille ICNS incorrecte")
    offset = 8
    while offset < len(data):
        require(offset + 8 <= len(data), "Bloc ICNS tronqué")
        size = struct.unpack_from(">I", data, offset + 4)[0]
        require(size > 8 and offset + size <= len(data), "Bloc ICNS invalide")
        offset += size


def read_zip(path: Path, required: set[str], optional: set[str], executables: set[str]) -> dict[str, bytes]:
    result: dict[str, bytes] = {}
    seen: set[str] = set()
    allowed = required | optional
    directories = {str(parent) for name in allowed for parent in PurePosixPath(name).parents if str(parent) != "."}
    total = 0
    with zipfile.ZipFile(path) as archive:
        require(len(archive.infolist()) <= MAX_MEMBERS, f"Trop d'entrées dans {path.name}")
        for member in archive.infolist():
            # Je contrôle le nom brut avant la normalisation propre à Windows de ZipInfo.
            name = safe_member(member.orig_filename)
            require(name not in seen, f"Entrée en double dans {path.name} : {name}")
            seen.add(name)
            mode = member.external_attr >> 16
            file_type = stat.S_IFMT(mode)
            require(file_type in {0, stat.S_IFDIR, stat.S_IFREG}, f"Lien ou fichier spécial interdit : {name}")
            require(member.flag_bits & 1 == 0, f"Archive chiffrée interdite : {name}")
            if member.is_dir():
                require(name in directories and member.file_size == 0, f"Dossier inattendu : {name}")
                if executables:
                    require(mode & 0o7022 == 0, f"Permissions de dossier non sûres : {name}")
                continue
            require(file_type != stat.S_IFDIR and name in allowed, f"Fichier inattendu : {name}")
            if executables:
                require(file_type == stat.S_IFREG and mode & 0o7022 == 0, f"Permissions de fichier non sûres : {name}")
            require(0 < member.file_size <= MAX_MEMBER_SIZE, f"Fichier vide ou trop volumineux : {name}")
            total += member.file_size
            require(total <= MAX_TOTAL_SIZE, "Archive décompressée trop volumineuse")
            if name in executables:
                require(mode & 0o111 == 0o111 and mode & 0o7022 == 0, f"Permissions exécutables non sûres : {name}")
            # Je lis jusqu'au bout pour déclencher également la vérification CRC du ZIP.
            result[name] = archive.read(member)
    require(required <= set(result), f"Fichiers requis absents de {path.name} : {sorted(required - set(result))}")
    return result


def verify_windows(distribution: Path) -> None:
    standalone = distribution / ARTIFACTS[0]
    with standalone.open("rb") as stream:
        check_pe(stream.read(1024 * 1024))
    files = read_zip(distribution / ARTIFACTS[1], {"EvoFarm.exe", "README.txt"}, LICENSES, set())
    require(hashlib.sha256(files["EvoFarm.exe"]).hexdigest() == digest_file(standalone),
            "L'exécutable du ZIP Windows diffère du téléchargement direct")


def verify_linux(distribution: Path) -> None:
    required = {f"EvoFarm/{name}" for name in (
        "evofarm", "README.txt", "logo.png", "fr.donj63000.evofarm.desktop", "install-desktop.sh"
    )}
    allowed = required | {f"EvoFarm/{name}" for name in LICENSES}
    result: dict[str, bytes] = {}
    seen: set[str] = set()
    total = 0
    # Je parcours aussi la fin du flux gzip pour contrôler son CRC, au-delà de la fin logique du tar.
    with gzip.open(distribution / ARTIFACTS[2], "rb") as compressed:
        decoded = 0
        for chunk in iter(lambda: compressed.read(1024 * 1024), b""):
            decoded += len(chunk)
            require(decoded <= MAX_TOTAL_SIZE, "Flux gzip décompressé trop volumineux")
    with tarfile.open(distribution / ARTIFACTS[2], "r:gz") as archive:
        for member in archive:
            require(len(seen) < MAX_MEMBERS, "Trop d'entrées dans l'archive Linux")
            name = safe_member(member.name)
            require(name not in seen, f"Entrée Linux en double : {name}")
            seen.add(name)
            require(member.isdir() or member.isfile(), f"Lien ou fichier spécial Linux interdit : {name}")
            if member.isdir():
                require(name == "EvoFarm", f"Dossier Linux inattendu : {name}")
                require(member.mode & 0o7022 == 0, f"Permissions de dossier Linux non sûres : {name}")
                continue
            require(name in allowed, f"Fichier Linux inattendu : {name}")
            require(0 < member.size <= MAX_MEMBER_SIZE, f"Taille Linux invalide : {name}")
            require(member.mode & 0o7022 == 0, f"Permissions Linux non sûres : {name}")
            if name in {"EvoFarm/evofarm", "EvoFarm/install-desktop.sh", "EvoFarm/fr.donj63000.evofarm.desktop"}:
                require(member.mode & 0o111 == 0o111, f"Permission d'exécution Linux absente : {name}")
            total += member.size
            require(total <= MAX_TOTAL_SIZE, "Archive Linux décompressée trop volumineuse")
            stream = archive.extractfile(member)
            require(stream is not None, f"Fichier Linux illisible : {name}")
            with stream:
                result[name] = stream.read()
            require(len(result[name]) == member.size, f"Fichier Linux tronqué : {name}")
    require(required <= set(result), f"Fichiers Linux requis absents : {sorted(required - set(result))}")
    check_elf(result["EvoFarm/evofarm"])
    require(result["EvoFarm/logo.png"].startswith(b"\x89PNG\r\n\x1a\n"), "Logo Linux PNG invalide")
    require(result["EvoFarm/install-desktop.sh"].startswith(b"#!/"), "Script d'installation Linux sans interpréteur")


def verify_macos(distribution: Path, version: str) -> None:
    required = {f"{BUNDLE}MacOS/evofarm", f"{BUNDLE}Info.plist", f"{BUNDLE}PkgInfo",
                f"{BUNDLE}Resources/EvoFarm.icns", f"{BUNDLE}_CodeSignature/CodeResources", "README.txt"}
    optional = LICENSES
    files = read_zip(distribution / ARTIFACTS[3], required, optional, {f"{BUNDLE}MacOS/evofarm"})
    check_macho(files[f"{BUNDLE}MacOS/evofarm"])
    check_icns(files[f"{BUNDLE}Resources/EvoFarm.icns"])
    require(files[f"{BUNDLE}PkgInfo"] == b"APPL????", "PkgInfo macOS invalide")
    metadata = plistlib.loads(files[f"{BUNDLE}Info.plist"])
    require(isinstance(metadata, dict), "Info.plist macOS n'est pas un dictionnaire")
    expected = {
        "CFBundleIdentifier": "fr.donj63000.evofarm",
        "CFBundleExecutable": "evofarm",
        "CFBundleName": "EvoFarm",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": version,
    }
    for key, value in expected.items():
        require(metadata.get(key) == value, f"Métadonnée macOS incorrecte : {key}")
    require(metadata.get("CFBundleIconFile") in {"EvoFarm", "EvoFarm.icns"}, "L'icône du bundle macOS n'est pas déclarée")


def verify_github(path: Path, distribution: Path, version: str) -> None:
    release = json.loads(path.read_text(encoding="utf-8-sig"))
    require(isinstance(release, dict), "Réponse GitHub invalide")
    require(release.get("tag_name") == f"v{version}", "Le tag GitHub ne correspond pas à la version")
    require(release.get("draft") is False and release.get("prerelease") is False, "La release GitHub n'est pas publique et stable")
    assets = release.get("assets")
    require(isinstance(assets, list), "Liste des téléchargements GitHub absente")
    expected = set(ARTIFACTS) | {"SHA256SUMS"}
    require(len(assets) == len(expected) and all(isinstance(asset, dict) for asset in assets),
            "Nombre de téléchargements GitHub incorrect")
    require(all(isinstance(asset.get("name"), str) for asset in assets), "Nom de téléchargement GitHub invalide")
    require({asset.get("name") for asset in assets} == expected, "Téléchargements GitHub inattendus ou manquants")
    for asset in assets:
        local = distribution / asset["name"]
        require(asset.get("size") == local.stat().st_size, f"Taille GitHub différente : {local.name}")
        require(asset.get("digest") == f"sha256:{digest_file(local)}", f"Empreinte GitHub différente : {local.name}")


def verify_release(distribution: Path, version: str, assemble: bool = False, github_json: Path | None = None) -> None:
    require(re.fullmatch(r"\d+\.\d+\.\d+", version) is not None, "Version attendue : X.Y.Z")
    require(distribution.is_dir(), "Dossier de distribution absent")
    allowed = set(ARTIFACTS) | {"SHA256SUMS", *INTERMEDIATE_MANIFESTS}
    for entry in distribution.iterdir():
        require(entry.name in allowed and entry.is_file() and not entry.is_symlink(),
                f"Élément inattendu dans la distribution : {entry.name}")
    for name in ARTIFACTS:
        path = distribution / name
        require(path.is_file() and not path.is_symlink() and 0 < path.stat().st_size <= MAX_TOTAL_SIZE,
                f"Livrable absent, vide ou invalide : {name}")
    if assemble:
        checksums = parse_manifest(distribution / "SHA256SUMS", set(ARTIFACTS[:2]))
        checksums.update(parse_manifest(distribution / INTERMEDIATE_MANIFESTS[0], {ARTIFACTS[2]}))
        checksums.update(parse_manifest(distribution / INTERMEDIATE_MANIFESTS[1], {ARTIFACTS[3]}))
    else:
        checksums = parse_manifest(distribution / "SHA256SUMS", set(ARTIFACTS))
    for name, checksum in checksums.items():
        require(digest_file(distribution / name) == checksum, f"Empreinte SHA-256 incorrecte : {name}")
    verify_windows(distribution)
    verify_linux(distribution)
    verify_macos(distribution, version)
    if assemble:
        # Je remplace le manifeste Windows seulement après validation de tous les livrables.
        contents = "".join(f"{checksums[name]}  {name}\n" for name in ARTIFACTS)
        pending: str | None = None
        try:
            with tempfile.NamedTemporaryFile(mode="w", encoding="ascii", newline="\n", dir=distribution,
                                             prefix=".SHA256SUMS-", delete=False) as stream:
                pending = stream.name
                stream.write(contents)
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(pending, distribution / "SHA256SUMS")
            pending = None
        finally:
            if pending is not None:
                Path(pending).unlink(missing_ok=True)
    if github_json is not None:
        verify_github(github_json, distribution, version)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--distribution", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--assemble", action="store_true", help="Fusionner les trois manifestes des plateformes après validation")
    parser.add_argument("--github-json", type=Path, help="Réponse JSON de l'API REST GitHub de la release publique")
    args = parser.parse_args()
    try:
        verify_release(args.distribution, args.version, args.assemble, args.github_json)
    except (ValidationError, OSError, UnicodeError, ValueError, KeyError, struct.error,
            zipfile.BadZipFile, tarfile.TarError, EOFError, plistlib.InvalidFileException) as error:
        print(f"Validation refusée : {error}", file=sys.stderr)
        return 1
    print(f"EvoFarm {args.version} : 4 livrables Windows, Linux et macOS vérifiés ; SHA-256 et archives valides.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
