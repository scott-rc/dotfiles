#!/usr/bin/env bash

set -euo pipefail

# --- Test Harness ---

PASS_COUNT=0
FAIL_COUNT=0
TEST_TMPDIR=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

setup() {
	TEST_TMPDIR="$(mktemp -d)"
}

teardown() {
	if [[ -n "$TEST_TMPDIR" && -d "$TEST_TMPDIR" ]]; then
		rm -rf "$TEST_TMPDIR"
	fi
}

pass() {
	local name="$1"
	echo "  PASS: $name"
	PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
	local name="$1"
	shift
	echo "  FAIL: $name"
	echo "        $*"
	FAIL_COUNT=$((FAIL_COUNT + 1))
}

assert_eq() {
	local test_name="$1"
	local expected="$2"
	local actual="$3"
	if [[ "$expected" == "$actual" ]]; then
		pass "$test_name"
	else
		fail "$test_name" "expected='$expected' actual='$actual'"
	fi
}

assert_link() {
	local test_name="$1"
	local link_path="$2"
	local expected_target="$3"
	if [[ -L "$link_path" ]]; then
		local actual_target
		actual_target="$(readlink "$link_path")"
		if [[ "$actual_target" == "$expected_target" ]]; then
			pass "$test_name"
		else
			fail "$test_name" "symlink exists but points to '$actual_target', expected '$expected_target'"
		fi
	else
		fail "$test_name" "'$link_path' is not a symlink"
	fi
}

assert_file_exists() {
	local test_name="$1"
	local path="$2"
	if [[ -e "$path" ]]; then
		pass "$test_name"
	else
		fail "$test_name" "'$path' does not exist"
	fi
}

assert_not_exists() {
	local test_name="$1"
	local path="$2"
	if [[ ! -e "$path" && ! -L "$path" ]]; then
		pass "$test_name"
	else
		fail "$test_name" "'$path' should not exist"
	fi
}

# --- Load library ---

LOG_LEVEL=error
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

# --- ensure_symlink tests ---

echo "ensure_symlink:"

# Test: creates a new symlink when target doesn't exist
setup
source_file="$TEST_TMPDIR/source.txt"
touch "$source_file"
target_link="$TEST_TMPDIR/target.txt"
ensure_symlink "$source_file" "$target_link"
assert_link "creates new symlink" "$target_link" "$source_file"
teardown

# Test: creates parent directories if needed
setup
source_file="$TEST_TMPDIR/source.txt"
touch "$source_file"
target_link="$TEST_TMPDIR/deep/nested/dir/target.txt"
ensure_symlink "$source_file" "$target_link"
assert_link "creates parent directories" "$target_link" "$source_file"
teardown

# Test: skips if symlink already points to correct source (idempotent)
setup
source_file="$TEST_TMPDIR/source.txt"
touch "$source_file"
target_link="$TEST_TMPDIR/target.txt"
ln -s "$source_file" "$target_link"
ensure_symlink "$source_file" "$target_link"
assert_link "idempotent — correct symlink unchanged" "$target_link" "$source_file"
assert_not_exists "idempotent — no .bak created" "$target_link.bak"
teardown

# Test: backs up existing regular file and creates symlink
setup
source_file="$TEST_TMPDIR/source.txt"
touch "$source_file"
target_link="$TEST_TMPDIR/target.txt"
echo "original content" > "$target_link"
ensure_symlink "$source_file" "$target_link"
assert_link "replaces regular file with symlink" "$target_link" "$source_file"
assert_file_exists "backs up regular file to .bak" "$target_link.bak"
assert_eq "backup has original content" "original content" "$(cat "$target_link.bak")"
teardown

# Test: replaces symlink pointing to wrong source
setup
source_file="$TEST_TMPDIR/source.txt"
wrong_source="$TEST_TMPDIR/wrong.txt"
touch "$source_file"
touch "$wrong_source"
target_link="$TEST_TMPDIR/target.txt"
ln -s "$wrong_source" "$target_link"
ensure_symlink "$source_file" "$target_link"
assert_link "replaces wrong symlink" "$target_link" "$source_file"
# The old symlink (pointing to wrong_source) gets backed up
assert_file_exists "backs up old symlink to .bak" "$target_link.bak"
teardown

# Test: backs up existing directory
setup
source_file="$TEST_TMPDIR/source.txt"
touch "$source_file"
target_link="$TEST_TMPDIR/target_dir"
mkdir -p "$target_link"
echo "inside dir" > "$target_link/file.txt"
ensure_symlink "$source_file" "$target_link"
assert_link "replaces directory with symlink" "$target_link" "$source_file"
assert_file_exists "backs up directory to .bak" "$target_link.bak"
assert_eq "backup directory has contents" "inside dir" "$(cat "$target_link.bak/file.txt")"
teardown

# Test: replaces dangling symlink (target points to non-existent source)
setup
source_file="$TEST_TMPDIR/source.txt"
touch "$source_file"
target_link="$TEST_TMPDIR/target.txt"
ln -s "$TEST_TMPDIR/deleted.txt" "$target_link"
ensure_symlink "$source_file" "$target_link"
assert_link "replaces dangling symlink" "$target_link" "$source_file"
assert_link "backs up dangling symlink to .bak" "$target_link.bak" "$TEST_TMPDIR/deleted.txt"
teardown

# --- Log function tests ---

echo ""
echo "log functions:"

# Log level test cases: [level_name, function, expected_output_at_level]
# We test that each function produces output at the right levels and suppresses at others.

# Helper to check if a log function produces output
log_produces_output() {
	local level="$1"
	local func="$2"
	local output
	LOG_LEVEL="$level"
	if [[ "$func" == "log_error" ]]; then
		output=$("$func" "test message" 2>&1)
	else
		output=$("$func" "test message" 2>/dev/null)
	fi
	LOG_LEVEL=error # restore
	[[ -n "$output" ]]
}

# Test matrix: [log_level, log_function, should_output]
test_cases=(
	"debug log_debug yes"
	"info  log_debug no"
	"warn  log_debug no"
	"error log_debug no"
	"debug log_info  yes"
	"info  log_info  yes"
	"warn  log_info  no"
	"error log_info  no"
	"debug log_warn  yes"
	"info  log_warn  yes"
	"warn  log_warn  yes"
	"error log_warn  no"
	"debug log_error yes"
	"info  log_error yes"
	"warn  log_error yes"
	"error log_error yes"
)

for case in "${test_cases[@]}"; do
	read -r level func expected <<< "$case"
	test_name="$func outputs at LOG_LEVEL=$level: $expected"
	if [[ "$expected" == "yes" ]]; then
		if log_produces_output "$level" "$func"; then
			pass "$test_name"
		else
			fail "$test_name" "expected output but got none"
		fi
	else
		if log_produces_output "$level" "$func"; then
			fail "$test_name" "expected no output but got some"
		else
			pass "$test_name"
		fi
	fi
done

# Test: log_error writes to stderr
setup
LOG_LEVEL=error
stderr_output=$(log_error "stderr test" 2>&1 1>/dev/null)
stdout_output=$(log_error "stdout test" 2>/dev/null)
LOG_LEVEL=error
if [[ -n "$stderr_output" && -z "$stdout_output" ]]; then
	pass "log_error writes to stderr, not stdout"
else
	fail "log_error writes to stderr, not stdout" "stderr='$stderr_output' stdout='$stdout_output'"
fi
teardown

# Test: _log_level_num returns correct values
assert_eq "_log_level_num debug = 0" "0" "$(_log_level_num debug)"
assert_eq "_log_level_num info = 1" "1" "$(_log_level_num info)"
assert_eq "_log_level_num warn = 2" "2" "$(_log_level_num warn)"
assert_eq "_log_level_num error = 3" "3" "$(_log_level_num error)"
assert_eq "_log_level_num unknown defaults to 1" "1" "$(_log_level_num unknown)"

# --- log_section tests ---

echo ""
echo "log_section:"

# Test: log_section outputs at LOG_LEVEL=info
LOG_LEVEL=info
output=$(log_section "Test Section" 2>/dev/null)
LOG_LEVEL=error
if [[ -n "$output" && "$output" == *"Test Section"* ]]; then
	pass "log_section outputs at LOG_LEVEL=info"
else
	fail "log_section outputs at LOG_LEVEL=info" "expected output containing 'Test Section', got '$output'"
fi

# Test: log_section suppressed at LOG_LEVEL=warn
LOG_LEVEL=warn
output=$(log_section "Test Section" 2>/dev/null)
LOG_LEVEL=error
if [[ -z "$output" ]]; then
	pass "log_section suppressed at LOG_LEVEL=warn"
else
	fail "log_section suppressed at LOG_LEVEL=warn" "expected no output but got '$output'"
fi

# --- log_success tests ---

echo ""
echo "log_success:"

# Test: log_success outputs at LOG_LEVEL=info
LOG_LEVEL=info
output=$(log_success "Done" 2>/dev/null)
LOG_LEVEL=error
if [[ -n "$output" && "$output" == *"Done"* ]]; then
	pass "log_success outputs at LOG_LEVEL=info"
else
	fail "log_success outputs at LOG_LEVEL=info" "expected output containing 'Done', got '$output'"
fi

# Test: log_success suppressed at LOG_LEVEL=warn
LOG_LEVEL=warn
output=$(log_success "Done" 2>/dev/null)
LOG_LEVEL=error
if [[ -z "$output" ]]; then
	pass "log_success suppressed at LOG_LEVEL=warn"
else
	fail "log_success suppressed at LOG_LEVEL=warn" "expected no output but got '$output'"
fi

# --- run_with_spinner tests ---

echo ""
echo "run_with_spinner:"

# Test: successful command shows checkmark
LOG_LEVEL=info
output=$(run_with_spinner "succeeds" true 2>&1)
LOG_LEVEL=error
if [[ "$output" == *"✔"* && "$output" == *"succeeds"* ]]; then
	pass "successful command shows checkmark"
else
	fail "successful command shows checkmark" "output='$output'"
fi

# Test: failed command returns non-zero exit code
LOG_LEVEL=info
exit_code=0
output=$(run_with_spinner "fails" false 2>&1) || exit_code=$?
LOG_LEVEL=error
if [[ $exit_code -ne 0 ]]; then
	pass "failed command returns non-zero exit code"
else
	fail "failed command returns non-zero exit code" "expected non-zero, got $exit_code"
fi

# Test: failed command shows X marker
if [[ "$output" == *"✖"* && "$output" == *"fails"* ]]; then
	pass "failed command shows X marker"
else
	fail "failed command shows X marker" "output='$output'"
fi

# Test: debug mode shows raw output
LOG_LEVEL=debug
output=$(run_with_spinner "echoing" echo "hello from debug" 2>&1)
LOG_LEVEL=error
if [[ "$output" == *"hello from debug"* && "$output" == *"✔"* ]]; then
	pass "debug mode shows raw output and checkmark"
else
	fail "debug mode shows raw output and checkmark" "output='$output'"
fi

# Test: captures and shows output on failure
LOG_LEVEL=info
exit_code=0
output=$(run_with_spinner "failing cmd" bash -c 'echo "error details"; exit 1' 2>&1) || exit_code=$?
LOG_LEVEL=error
if [[ $exit_code -ne 0 && "$output" == *"error details"* ]]; then
	pass "captures and shows output on failure"
else
	fail "captures and shows output on failure" "exit_code=$exit_code output='$output'"
fi

# Test: silent mode (warn) runs without output
LOG_LEVEL=warn
output=$(run_with_spinner "silent" echo "should not appear" 2>&1)
LOG_LEVEL=error
if [[ -z "$output" ]]; then
	pass "silent mode produces no output"
else
	fail "silent mode produces no output" "output='$output'"
fi

# --- Summary ---

echo ""
total=$((PASS_COUNT + FAIL_COUNT))
echo "Results: $PASS_COUNT/$total passed, $FAIL_COUNT failed"
if [[ "$FAIL_COUNT" -gt 0 ]]; then
	exit 1
fi
