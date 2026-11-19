#!/usr/bin/env python3
"""Fail when unit-testable production coverage is below a threshold.

Usage:
    python3 scripts/coverage_gate.py target/coverage/lcov.info [80]

The report must be lcov produced by:

    cargo llvm-cov --lcov --output-path target/coverage/lcov.info

Scope: src/core, src/application, src/presentation, plus the pure adapter
helpers src/adapters/window.rs, src/adapters/ui/theme.rs and
src/adapters/ui/cursor.rs. The eframe/egui shell (src/main.rs, app.rs and the
painting modules) needs a live window or frame, so it is excluded. Inline
#[cfg(test)] modules are excluded from the measured lines.
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"

SCOPE_PREFIXES = ("core/", "application/", "presentation/", "adapters/")

def in_scope(relative_path: str) -> bool:
    return relative_path.startswith(SCOPE_PREFIXES)


def area_of(relative_path: str) -> str:
    return relative_path.split("/", 1)[0]


def parse_lcov(path: Path) -> dict[Path, dict[int, int]]:
    """Maps each source file to {line number: execution count}."""
    files: dict[Path, dict[int, int]] = {}
    current: Path | None = None
    for raw in path.read_text().splitlines():
        if raw.startswith("SF:"):
            current = Path(raw[3:]).resolve()
            files.setdefault(current, {})
        elif raw.startswith("DA:") and current is not None:
            line, count = raw[3:].split(",")[:2]
            line_number = int(line)
            files[current][line_number] = max(
                files[current].get(line_number, 0), int(count)
            )
    return files


def first_test_line(path: Path) -> int | None:
    """Line of the first #[cfg(test)] in the file, if any.

    Unit tests in this project live at the bottom of the file, so everything
    from that line onwards is test code and must not count as production.
    """
    with path.open(encoding="utf-8") as source:
        for number, line in enumerate(source, 1):
            if "#[cfg(test)]" in line:
                return number
    return None


def main(argv: list[str]) -> int:
    if len(argv) not in (2, 3):
        print(f"usage: {argv[0]} <lcov.info> [threshold]", file=sys.stderr)
        return 2

    report = Path(argv[1])
    threshold = float(argv[2]) if len(argv) == 3 else 80.0
    if not report.is_file():
        print(f"coverage report not found: {report}", file=sys.stderr)
        return 2

    per_area: dict[str, list[int]] = {}
    for path, lines in parse_lcov(report).items():
        try:
            relative = path.relative_to(SRC.resolve()).as_posix()
        except ValueError:
            continue
        if not in_scope(relative):
            continue

        cutoff = first_test_line(path)
        production = [
            count
            for line, count in lines.items()
            if cutoff is None or line < cutoff
        ]
        if not production:
            continue

        area = per_area.setdefault(area_of(relative), [0, 0])
        area[0] += len(production)
        area[1] += sum(1 for count in production if count > 0)

    total_found = sum(found for found, _ in per_area.values())
    total_hit = sum(hit for _, hit in per_area.values())
    if total_found == 0:
        print("no in-scope production lines in the report", file=sys.stderr)
        return 2

    print("Unit-testable production coverage (inline tests excluded)")
    for area, (found, hit) in sorted(per_area.items()):
        print(f"  {area:<14}{hit:>5}/{found:<5} {100 * hit / found:>7.2f}%")
    percentage = 100 * total_hit / total_found
    print(f"  {'total':<14}{total_hit:>5}/{total_found:<5} {percentage:>7.2f}%")

    verdict = "PASS" if percentage >= threshold else "FAIL"
    print(f"\nCoverage gate: {threshold:.2f}% -> {verdict}")
    return 0 if verdict == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
