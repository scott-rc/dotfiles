#!/bin/bash
# Creates a test git repo with staged, unstaged, and untracked changes.
# Run before tests to reset to known state.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$SCRIPT_DIR/test-repo"

rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"

git init -q
git config user.email "test@example.com"
git config user.name "Test User"

# Initial commit with some files
cat > main.rs << 'EOF'
fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x + 1;
    println!("Result: {}", y);
}
EOF

cat > lib.rs << 'EOF'
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}
EOF

mkdir -p src/utils
cat > src/utils/helpers.rs << 'EOF'
pub fn format_number(n: i32) -> String {
    format!("{}", n)
}
EOF

git add .
git commit -q -m "Initial commit"

# Staged change: modify main.rs
cat > main.rs << 'EOF'
fn main() {
    println!("Hello, gd!");
    let x = 100;
    let y = x + 1;
    println!("Result: {}", y);
    println!("Done!");
}
EOF
git add main.rs

# Unstaged change: modify lib.rs
cat > lib.rs << 'EOF'
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}

pub fn mul(a: i32, b: i32) -> i32 {
    a * b
}
EOF

# Untracked file
cat > new_file.rs << 'EOF'
// This is a new untracked file
pub fn new_function() {
    println!("I'm new!");
}
EOF

echo "Test repo created at $REPO_DIR"
git status
