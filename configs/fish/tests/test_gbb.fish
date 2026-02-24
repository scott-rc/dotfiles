#!/usr/bin/env fish

# Regression tests for gbb (git base branch detection)

source (status dirname)/../functions/gbb.fish

set -g test_passes 0
set -g test_failures 0

function assert_eq -a label expected actual
    if test "$expected" = "$actual"
        set test_passes (math $test_passes + 1)
        echo "  PASS: $label"
    else
        set test_failures (math $test_failures + 1)
        echo "  FAIL: $label"
        echo "    expected: '$expected'"
        echo "    actual:   '$actual'"
    end
end

function setup_repo -a dir
    set -l bare "$dir.bare"
    git init --bare -b main $bare >/dev/null 2>&1
    git clone $bare $dir >/dev/null 2>&1
    cd $dir
    git config user.email "test@test.com"
    git config user.name "Test"
end

function push_main
    git push -u origin main >/dev/null 2>&1
    git remote set-head origin main >/dev/null 2>&1
end

set -l tmpdir (mktemp -d)

# ─── stale branch on main line should not be returned ────────────────────
#
#   A -- B -- C -- D (main)
#         \
#          E -- F (feature)
#   stale-branch tip at B (ancestor of main)
#
#   gbb on feature should return "main", not "stale-branch"

echo "test: stale branch ancestor of default branch"
setup_repo "$tmpdir/stale"
git commit --allow-empty -m "A" >/dev/null 2>&1
git commit --allow-empty -m "B" >/dev/null 2>&1
git branch stale-branch >/dev/null 2>&1
git commit --allow-empty -m "C" >/dev/null 2>&1
git commit --allow-empty -m "D" >/dev/null 2>&1
push_main
git checkout -b feature stale-branch >/dev/null 2>&1
git commit --allow-empty -m "E" >/dev/null 2>&1
git commit --allow-empty -m "F" >/dev/null 2>&1
assert_eq "returns default branch, not stale" "main" (gbb)

# ─── legitimate intermediate branch (not ancestor of default) ────────────
#
#   A -- B -- C (main)
#         \
#          D -- E (intermediate)
#               \
#                F -- G (feature)
#
#   gbb on feature should return "intermediate"

echo "test: intermediate branch not ancestor of default"
setup_repo "$tmpdir/intermediate"
git commit --allow-empty -m "A" >/dev/null 2>&1
git commit --allow-empty -m "B" >/dev/null 2>&1
git commit --allow-empty -m "C" >/dev/null 2>&1
push_main
git checkout -b intermediate HEAD~1 >/dev/null 2>&1
git commit --allow-empty -m "D" >/dev/null 2>&1
git commit --allow-empty -m "E" >/dev/null 2>&1
git checkout -b feature >/dev/null 2>&1
git commit --allow-empty -m "F" >/dev/null 2>&1
git commit --allow-empty -m "G" >/dev/null 2>&1
assert_eq "returns intermediate branch" "intermediate" (gbb)

# ─── simple feature off main (main tip on the walk) ─────────────────────
#
#   A -- B (main)
#         \
#          C -- D (feature)
#
#   gbb on feature should return "main"

echo "test: feature directly off main"
setup_repo "$tmpdir/simple"
git commit --allow-empty -m "A" >/dev/null 2>&1
git commit --allow-empty -m "B" >/dev/null 2>&1
push_main
git checkout -b feature >/dev/null 2>&1
git commit --allow-empty -m "C" >/dev/null 2>&1
git commit --allow-empty -m "D" >/dev/null 2>&1
assert_eq "returns main" "main" (gbb)

# ─── on default branch returns itself ────────────────────────────────────

echo "test: on default branch"
setup_repo "$tmpdir/default"
git commit --allow-empty -m "A" >/dev/null 2>&1
push_main
assert_eq "returns main when on main" "main" (gbb)

# ─── cleanup ─────────────────────────────────────────────────────────────

cd /
rm -rf $tmpdir

echo ""
echo "$test_passes passed, $test_failures failed"
exit $test_failures
