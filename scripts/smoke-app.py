"""Je vérifie le démarrage réel du programme sur un runner CI temporaire."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


def check_macos_dependencies(binary: Path) -> None:
    libraries = subprocess.run(
        ["otool", "-L", str(binary)], check=True, capture_output=True, text=True
    ).stdout
    for line in libraries.splitlines():
        if " (compatibility version " not in line:
            continue
        library = line.strip().split(" (compatibility version ", 1)[0]
        if not library.startswith(("/System/Library/", "/usr/lib/")):
            raise RuntimeError(f"Dépendance macOS extérieure au système : {library}")
    commands = subprocess.run(
        ["otool", "-l", str(binary)], check=True, capture_output=True, text=True
    ).stdout
    minima = re.findall(r"cmd LC_BUILD_VERSION\s+cmdsize \d+\s+platform \S+\s+minos ([0-9.]+)", commands)
    minima += re.findall(r"cmd LC_VERSION_MIN_MACOSX\s+cmdsize \d+\s+version ([0-9.]+)", commands)
    if not minima:
        raise RuntimeError("Version minimale de macOS absente du binaire.")
    for minimum in minima:
        if tuple(int(part) for part in minimum.split(".")[:2]) > (11, 0):
            raise RuntimeError(f"Le binaire exige macOS {minimum}, au-delà de 11.0.")
    print(libraries.strip())


def check_startup(command: list[str], duration: float = 5.0) -> str:
    if duration <= 0:
        raise ValueError("La durée du contrôle doit être positive.")
    # Je termine uniquement le processus que ce contrôle vient de créer.
    with tempfile.TemporaryFile() as output:
        process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=output, stderr=output)
        early_exit = False
        try:
            try:
                process.wait(timeout=duration)
                early_exit = True
            except subprocess.TimeoutExpired:
                pass
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
        output.seek(0)
        log = output.read().decode("utf-8", errors="replace")
    if early_exit:
        raise RuntimeError(f"Le programme s'est arrêté au démarrage (code {process.returncode}).\n{log}")
    return log


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--check-macos", action="store_true")
    args = parser.parse_args()
    # Je réserve le lancement aux runners jetables ; leurs profils ne contiennent aucune sauvegarde utilisateur.
    if os.environ.get("CI", "").lower() != "true":
        parser.error("Ce test de lancement est réservé à un runner CI temporaire.")
    binary = args.binary.resolve(strict=True)
    if args.check_macos:
        check_macos_dependencies(binary)
    log = check_startup([str(binary)])
    if log.strip():
        print(log.rstrip())
    print("Démarrage réussi : le programme reste actif pendant cinq secondes.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
