#!/usr/bin/env python3
"""Exécute les vérifications natives, sans confondre un contrôle absent et réussi.

Python 3.11+, Rust/rustfmt/Clippy et cargo-audit sont requis. Aucun shell n'est
utilisé pour interpoler des arguments. Les résultats sont conservés sous target/.
L'option --format autorise explicitement rustfmt à modifier les fichiers sources.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def validation_commands(cargo: str, apply_format: bool) -> list[tuple[str, list[str]]]:
    """La même séquence s'exécute sur Windows, Linux et macOS."""
    steps: list[tuple[str, list[str]]] = []
    if apply_format:
        steps.append(("format-apply", [cargo, "fmt", "--all"]))
    steps.extend([
        ("format-check", [cargo, "fmt", "--all", "--", "--check"]),
        ("clippy", [cargo, "clippy", "--all-targets", "--locked", "--", "-D", "warnings"]),
        ("rust-tests", [cargo, "test", "--all-targets", "--locked"]),
        ("python-tests", [sys.executable, "-m", "unittest", "discover", "-s", "scripts", "-p", "test_*.py", "-v"]),
        ("dependency-audit", [cargo, "audit", "--json"]),
    ])
    return steps


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", action="store_true", help="Appliquer rustfmt avant les contrôles")
    args = parser.parse_args()
    output = ROOT / "target" / "security-validation"
    output.mkdir(parents=True, exist_ok=True)
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    results: list[dict[str, object]] = []
    cargo = shutil.which("cargo")
    if cargo is None:
        results.append({"step": "toolchain", "returncode": 127, "status": "NOT_RUN", "reason": "cargo introuvable"})
        print("Validation native NON exécutée : cargo est introuvable.", file=sys.stderr)
    else:
        for name, command in validation_commands(cargo, args.format):
            print(f"\n== {name} ==", flush=True)
            log = output / f"{name}.log"
            try:
                with log.open("wb") as stream:
                    process = subprocess.run(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, check=False)
                code = process.returncode
            except OSError as error:
                log.write_text(str(error), encoding="utf-8")
                code = 127
            result = {"step": name, "command": command, "returncode": code, "status": "PASS" if code == 0 else "FAIL"}
            results.append(result)
            print(f"{result['status']} — détails : {log}")
            if code != 0:
                # Aucune validation globale après une étape échouée ou indisponible.
                break
    complete = len(results) == len(validation_commands(cargo or "cargo", args.format))
    passed = complete and all(item["returncode"] == 0 for item in results)
    report = {"started_at_utc": started, "complete": complete, "passed": passed, "results": results}
    (output / "result.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print("Validation complète réussie." if passed else "Validation incomplète ou en échec : ne pas publier sur ce résultat.")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
