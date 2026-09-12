#!/usr/bin/env python3
"""Je prépare les distributions natives Linux et macOS avec Python 3.11+."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import plistlib
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile


LINUX_TARGET = "x86_64-unknown-linux-gnu"
MACOS_TARGETS = ("x86_64-apple-darwin", "aarch64-apple-darwin")
MACOS_MINIMUM = "11.0"
ARCHIVES = {
    "linux": "EvoFarm-Linux-x64.tar.gz",
    "macos": "EvoFarm-macOS-universal.zip",
}


class BuildError(RuntimeError):
    """Je signale un échec de préparation sans remplacer la distribution précédente."""


def run_command(command: list[str], *, cwd: Path, env: dict[str, str]) -> None:
    try:
        subprocess.run(command, cwd=cwd, env=env, check=True)
    except (OSError, subprocess.CalledProcessError) as error:
        raise BuildError(f"Échec de l'outil {command[0]} : {error}") from error


def require_file(path: Path) -> None:
    # Je refuse les liens et les fichiers vides pour limiter le contenu aux produits attendus.
    if path.is_symlink() or not path.is_file() or path.stat().st_size == 0:
        raise BuildError(f"Fichier requis absent, vide ou symbolique : {path}")


def package_version(project: Path) -> str:
    require_file(project / "Cargo.toml")
    with (project / "Cargo.toml").open("rb") as manifest:
        package = tomllib.load(manifest)["package"]
    version = package["version"]
    if package["name"] != "evofarm" or not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise BuildError("Cargo.toml doit décrire evofarm avec une version de publication X.Y.Z.")
    return version


def target_directory(project: Path, env: dict[str, str]) -> Path:
    value = Path(env.get("CARGO_TARGET_DIR") or "target").expanduser()
    return (value if value.is_absolute() else project / value).resolve()


def copy_file(source: Path, destination: Path, mode: int = 0o644) -> None:
    require_file(source)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(mode)


def desktop_entry() -> str:
    # Je passe le chemin du lanceur comme argument, jamais comme du code interprété.
    shell_command = 'test -n "$1" && exec "$(dirname -- "$1")/evofarm"'
    quoted = "".join("\\" + char if char in '\\"`$' else char for char in shell_command)
    quoted = quoted.replace("\\", "\\\\")
    return (
        "[Desktop Entry]\n"
        "Type=Application\n"
        "Name=EvoFarm\n"
        "Comment=Compagnon de farm pour Dofus Retro\n"
        f'Exec=sh -c "{quoted}" evofarm %k\n'
        "Icon=fr.donj63000.evofarm\n"
        "Terminal=false\n"
        "Categories=Game;Utility;\n"
        "StartupWMClass=fr.donj63000.evofarm\n"
    )


def desktop_installer() -> str:
    return r'''#!/bin/sh
set -eu

# Je référence le dossier portable sans déplacer le programme ni ses données.
app_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
data_dir=${XDG_DATA_HOME:-"$HOME/.local/share"}
case "$data_dir" in
    /*) ;;
    *) printf '%s\n' 'XDG_DATA_HOME doit être un chemin absolu.' >&2; exit 1 ;;
esac
newline='
'
case "$app_dir" in
    *"$newline"*) printf '%s\n' 'Un retour à la ligne dans le chemin du programme est incompatible avec ce raccourci.' >&2; exit 1 ;;
esac
test -x "$app_dir/evofarm"
test -f "$app_dir/logo.png"

# Je protège les caractères réservés du format Desktop Entry et les codes de champ.
executable=$(printf '%s' "$app_dir/evofarm" | sed 's/\\/\\\\/g; s/["`$]/\\&/g; s/\\/\\\\/g; s/%/%%/g')
exec_command='sh -c "exec \\"\\$1\\"" evofarm'
applications="$data_dir/applications"
icons="$data_dir/icons/hicolor/512x512/apps"
mkdir -p -- "$applications" "$icons"
install -m 644 -- "$app_dir/logo.png" "$icons/fr.donj63000.evofarm.png"
entry=$(mktemp "$applications/.evofarm.XXXXXX")
trap 'rm -f -- "$entry"' EXIT HUP INT TERM
cat > "$entry" <<EOF
[Desktop Entry]
Type=Application
Name=EvoFarm
Comment=Compagnon de farm pour Dofus Retro
Exec=$exec_command "$executable"
Icon=fr.donj63000.evofarm
Terminal=false
Categories=Game;Utility;
StartupWMClass=fr.donj63000.evofarm
EOF
chmod 644 "$entry"
mv -f -- "$entry" "$applications/fr.donj63000.evofarm.desktop"
trap - EXIT HUP INT TERM
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$applications" || true
fi
printf '%s\n' 'Raccourci EvoFarm installé pour cet utilisateur.' "Conservez le dossier du programme à cet emplacement : $app_dir"
'''


def create_linux_archive(stage: Path, archive: Path) -> None:
    # Je fixe les permissions et les propriétaires sans exposer les comptes du constructeur.
    with tarfile.open(archive, "w:gz", format=tarfile.PAX_FORMAT) as output:
        for path in sorted(stage.rglob("*")):
            relative = path.relative_to(stage).as_posix()
            info = output.gettarinfo(str(path), arcname=relative)
            if not (info.isfile() or info.isdir()):
                raise BuildError(f"Entrée non autorisée dans l'archive : {relative}")
            info.uid = info.gid = 0
            info.uname = info.gname = ""
            info.mtime = 0
            info.mode = 0o755 if path.is_dir() or path.name in {"evofarm", "fr.donj63000.evofarm.desktop", "install-desktop.sh"} else 0o644
            if path.is_file():
                with path.open("rb") as source:
                    output.addfile(info, source)
            else:
                output.addfile(info)


def create_macos_archive(stage: Path, archive: Path) -> None:
    # Je conserve les permissions Unix dans le ZIP, notamment le droit de lancer l'application.
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as output:
        for path in sorted(stage.rglob("*")):
            if path.is_symlink() or not (path.is_file() or path.is_dir()):
                raise BuildError(f"Entrée non autorisée dans l'archive : {path}")
            relative = path.relative_to(stage).as_posix()
            if path.is_dir():
                relative += "/"
            entry = zipfile.ZipInfo(relative)
            entry.create_system = 3
            mode = (stat.S_IFDIR | 0o755) if path.is_dir() else (stat.S_IFREG | (0o755 if path.name == "evofarm" else 0o644))
            entry.external_attr = mode << 16
            if path.is_dir():
                entry.external_attr |= 0x10
            entry.compress_type = zipfile.ZIP_DEFLATED
            output.writestr(entry, b"" if path.is_dir() else path.read_bytes())


def prepare_macos_bundle(project: Path, stage: Path, scratch: Path, version: str,
                         targets: Path, env: dict[str, str]) -> None:
    contents = stage / "EvoFarm.app" / "Contents"
    executable = contents / "MacOS" / "evofarm"
    executable.parent.mkdir(parents=True)
    resources = contents / "Resources"
    resources.mkdir()
    binaries = [targets / target / "release" / "evofarm" for target in MACOS_TARGETS]
    for binary in binaries:
        require_file(binary)
    run_command(["lipo", "-create", *(str(binary) for binary in binaries), "-output", str(executable)], cwd=project, env=env)
    require_file(executable)
    executable.chmod(0o755)
    # Je place le fichier avant la liste d'architectures, qui consomme tous les arguments suivants.
    run_command(["lipo", str(executable), "-verify_arch", "x86_64", "arm64"], cwd=project, env=env)

    iconset = scratch / "EvoFarm.iconset"
    iconset.mkdir()
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            suffix = "@2x" if scale == 2 else ""
            icon = iconset / f"icon_{size}x{size}{suffix}.png"
            run_command(["sips", "-s", "format", "png", "-z", str(size * scale), str(size * scale),
                         str(project / "logo.png"), "--out", str(icon)], cwd=project, env=env)
            require_file(icon)
    app_icon = resources / "EvoFarm.icns"
    run_command(["iconutil", "-c", "icns", "-o", str(app_icon), str(iconset)], cwd=project, env=env)
    require_file(app_icon)

    metadata = {
        "CFBundleDevelopmentRegion": "fr",
        "CFBundleDisplayName": "EvoFarm",
        "CFBundleExecutable": "evofarm",
        "CFBundleIconFile": "EvoFarm.icns",
        "CFBundleIdentifier": "fr.donj63000.evofarm",
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": "EvoFarm",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": version,
        "LSMinimumSystemVersion": env["MACOSX_DEPLOYMENT_TARGET"],
        "NSHighResolutionCapable": True,
        "NSHumanReadableCopyright": "Dev By Donj63000(Coca)",
    }
    with (contents / "Info.plist").open("wb") as plist:
        plistlib.dump(metadata, plist, sort_keys=True)
    (contents / "PkgInfo").write_bytes(b"APPL????")
    bundle = stage / "EvoFarm.app"
    # Je signe localement les deux architectures ; cette signature ne remplace pas une notarisation Apple.
    run_command(["codesign", "--force", "--sign", "-", "--timestamp=none", str(bundle)], cwd=project, env=env)
    run_command(["codesign", "--verify", "--deep", "--strict", "--all-architectures", str(bundle)], cwd=project, env=env)


def build_release(project: Path, platform: str, output_dir: Path | None = None,
                  environment: dict[str, str] | None = None) -> tuple[Path, Path]:
    if platform not in ARCHIVES:
        raise BuildError(f"Plateforme inconnue : {platform}")
    project = project.resolve()
    version = package_version(project)
    for asset in ("logo.png", "fond.png", "README.txt"):
        require_file(project / asset)
    env = dict(os.environ if environment is None else environment)
    if platform == "macos":
        minimum = env.get("MACOSX_DEPLOYMENT_TARGET") or MACOS_MINIMUM
        if not re.fullmatch(r"\d+\.\d+(?:\.\d+)?", minimum) or int(minimum.split(".")[0]) < 11:
            raise BuildError("MACOSX_DEPLOYMENT_TARGET doit désigner macOS 11.0 ou une version ultérieure.")
        env["MACOSX_DEPLOYMENT_TARGET"] = minimum
    targets = target_directory(project, env)
    triples = (LINUX_TARGET,) if platform == "linux" else MACOS_TARGETS
    for target in triples:
        run_command(["cargo", "build", "--manifest-path", str(project / "Cargo.toml"),
                     "--locked", "--release", "--target", target], cwd=project, env=env)
        require_file(targets / target / "release" / "evofarm")

    destination = (output_dir or project / "dist").resolve()
    destination.mkdir(parents=True, exist_ok=True)
    archive_path = destination / ARCHIVES[platform]
    checksums_path = destination / f"SHA256SUMS-{platform}"
    # Je construis dans un dossier temporaire du même volume et ne remplace que des produits complets.
    with tempfile.TemporaryDirectory(prefix=f".evofarm-{platform}-", dir=destination) as temporary:
        scratch = Path(temporary)
        stage = scratch / "payload"
        stage.mkdir()
        documents = stage / "EvoFarm" if platform == "linux" else stage
        copy_file(project / "README.txt", documents / "README.txt")
        for optional in ("LICENSE", "LICENSE.txt", "LICENSE.md"):
            if (project / optional).exists():
                copy_file(project / optional, documents / optional)
        if platform == "linux":
            copy_file(targets / LINUX_TARGET / "release" / "evofarm", documents / "evofarm", 0o755)
            copy_file(project / "logo.png", documents / "logo.png")
            launcher = documents / "fr.donj63000.evofarm.desktop"
            launcher.write_text(desktop_entry(), encoding="utf-8", newline="\n")
            launcher.chmod(0o755)
            installer = documents / "install-desktop.sh"
            installer.write_text(desktop_installer(), encoding="utf-8", newline="\n")
            installer.chmod(0o755)
        else:
            prepare_macos_bundle(project, stage, scratch, version, targets, env)
        pending_archive = scratch / archive_path.name
        if platform == "linux":
            create_linux_archive(stage, pending_archive)
        else:
            create_macos_archive(stage, pending_archive)
        require_file(pending_archive)
        with pending_archive.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        pending_checksums = scratch / checksums_path.name
        pending_checksums.write_text(f"{digest}  {archive_path.name}\n", encoding="ascii", newline="\n")
        os.replace(pending_archive, archive_path)
        os.replace(pending_checksums, checksums_path)
    return archive_path, checksums_path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", choices=tuple(ARCHIVES), required=True)
    parser.add_argument("--output-dir", type=Path, help="Dossier des archives (par défaut : dist/).")
    args = parser.parse_args()
    expected_host = "linux" if args.platform == "linux" else "darwin"
    if sys.platform != expected_host:
        parser.error(f"La distribution {args.platform} doit être construite sur {expected_host}.")
    try:
        archive, checksums = build_release(Path(__file__).resolve().parent.parent, args.platform, args.output_dir)
    except (BuildError, OSError, ValueError, KeyError) as error:
        print(f"Publication interrompue : {error}", file=sys.stderr)
        return 1
    print(f"Archive créée : {archive}")
    print(f"Empreinte créée : {checksums}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
