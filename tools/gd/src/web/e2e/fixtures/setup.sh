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

# File with many context lines between changes (for collapsible context testing)
cat > long_file.rs << 'EOF'
use std::io;

pub struct Config {
    pub name: String,
    pub version: u32,
    pub debug: bool,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            name: String::from("default"),
            version: 1,
            debug: false,
            max_retries: 3,
            timeout_ms: 5000,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.is_empty() && self.timeout_ms > 0
    }

    pub fn display(&self) {
        println!("Config: {} v{}", self.name, self.version);
    }
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

# Unstaged change: modify long_file.rs (changes spread far apart, creates collapsible context)
cat > long_file.rs << 'EOF'
use std::io;
use std::fmt;

pub struct Config {
    pub name: String,
    pub version: u32,
    pub debug: bool,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            name: String::from("default"),
            version: 1,
            debug: true,
            max_retries: 3,
            timeout_ms: 5000,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.is_empty() && self.timeout_ms > 0
    }

    pub fn display(&self) {
        println!("Config: {} v{} (debug={})", self.name, self.version, self.debug);
    }
}
EOF

# Unstaged change: modify lib.rs (includes deletion and addition)
cat > lib.rs << 'EOF'
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
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

# Add files with different extensions for icon testing
cat > script.js << 'EOF'
// JavaScript file for testing
function hello() {
    console.log("Hello from JS");
}
EOF

cat > README.md << 'EOF'
# Test README

This is a markdown file for testing.
EOF

# Add nested directory structure for collapse testing
mkdir -p src/components/ui
cat > src/components/button.rs << 'EOF'
// Button component
pub struct Button {}
EOF

cat > src/components/ui/modal.rs << 'EOF'
// Modal component
pub struct Modal {}
EOF

cat > src/components/ui/tooltip.rs << 'EOF'
// Tooltip component
pub struct Tooltip {}
EOF

echo "Test repo created at $REPO_DIR"
git status
