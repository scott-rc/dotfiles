#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

case "${1:-html}" in
  html)  cargo llvm-cov --package gd --html --open ;;
  lcov)  cargo llvm-cov --package gd --lcov --output-path lcov.info ;;
  text)  cargo llvm-cov --package gd --text ;;
  json)  cargo llvm-cov --package gd --json --output-path coverage.json ;;
  agent)
    cargo llvm-cov --package gd --lcov --output-path lcov.info

    python3 - lcov.info <<'PYEOF'
import json, subprocess, sys, os

lcov_path = sys.argv[1]
tool_root = "/tools/gd/"

# Reuse coverage data (no-run) to get JSON summary + function details
json_raw = subprocess.check_output(
    ["cargo", "llvm-cov", "--package", "gd", "--json", "--no-run"],
    stderr=subprocess.DEVNULL,
)
d = json.loads(json_raw)

# --- COVERAGE SUMMARY ---
print("\n=== COVERAGE SUMMARY ===")
print(f"{'File':<55} {'Lines':>6} {'Missed':>8} {'Cover':>7}")
print("-" * 80)
for f in sorted(d["data"][0]["files"], key=lambda f: f["filename"]):
    name = f["filename"]
    if tool_root in name:
        name = name.split(tool_root)[-1]
    elif "/tools/" in name:
        continue
    if "/tests/" in name:
        continue
    s = f["summary"]["lines"]
    missed = s["count"] - s["covered"]
    print(f"{name:<55} {s['count']:>6} {missed:>8} {s['percent']:>6.1f}%")
totals = d["data"][0]["totals"]["lines"]
print("-" * 80)
missed = totals["count"] - totals["covered"]
print(f"{'TOTAL':<55} {totals['count']:>6} {missed:>8} {totals['percent']:>6.1f}%")

# --- UNCOVERED FUNCTIONS (from JSON, with real line numbers) ---
print("\n=== UNCOVERED FUNCTIONS ===")
raw_lines = []
for func in d["data"][0]["functions"]:
    if func["count"] != 0:
        continue
    fname = func["filenames"][0] if func["filenames"] else "?"
    if "/tests/" in fname or "/tools/tui/" in fname:
        continue
    if ".cargo/registry" in fname:
        continue
    start_line = func["regions"][0][0] if func["regions"] else 0
    if tool_root in fname:
        fname = fname.split(tool_root)[-1]
    raw_lines.append(f"{fname}:{start_line} {func['name']}")

# Demangle if rustfilt available
text = "\n".join(raw_lines)
try:
    result = subprocess.run(
        ["rustfilt"], input=text, capture_output=True, text=True, timeout=5
    )
    if result.returncode == 0:
        text = result.stdout.rstrip()
except (FileNotFoundError, subprocess.TimeoutExpired):
    pass

seen = set()
for line in sorted(text.splitlines(), key=lambda l: (l.split(":")[0], int(l.split(":")[1].split()[0]))):
    if "{closure" in line:
        continue
    if line not in seen:
        seen.add(line)
        print(line)

# --- UNCOVERED LINES BY FILE (from lcov) ---
print("\n=== UNCOVERED LINES BY FILE ===")
current_file = None
uncovered = []

def flush(f, lines):
    if not f or not lines or "/tests/" in f:
        return
    lines.sort()
    ranges = []
    rs = re = lines[0]
    for l in lines[1:]:
        if l == re + 1:
            re = l
        else:
            ranges.append(str(rs) if rs == re else f"{rs}-{re}")
            rs = re = l
    ranges.append(str(rs) if rs == re else f"{rs}-{re}")
    name = f.split(tool_root)[-1] if tool_root in f else f
    print(f"{name} (uncovered: {','.join(ranges)})")

for raw in open(lcov_path):
    line = raw.rstrip()
    if line.startswith("SF:"):
        if current_file:
            flush(current_file, uncovered)
        current_file = line[3:]
        uncovered = []
    elif line.startswith("DA:"):
        parts = line[3:].split(",")
        if parts[1] == "0":
            uncovered.append(int(parts[0]))
    elif line == "end_of_record":
        flush(current_file, uncovered)
        current_file = None
        uncovered = []
PYEOF
    ;;
  *)
    echo "Usage: $0 [html|lcov|text|json|agent]"
    echo ""
    echo "  html   Open HTML report in browser (default)"
    echo "  lcov   Write lcov.info for IDE gutter integration"
    echo "  text   Print terminal summary table"
    echo "  json   Write coverage.json (LLVM export format)"
    echo "  agent  Write lcov.info + print uncovered functions/lines"
    exit 1
    ;;
esac
