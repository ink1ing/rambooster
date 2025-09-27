#!/bin/bash
# scripts/check_rust_toolchain.sh

# Function to check for a command
check_command() {
  if ! command -v "$1" &> /dev/null; then
    echo "Error: $1 is not installed. Please install it."
    exit 1
  fi
}

# Function to check for a cargo component
check_cargo_component() {
  if ! cargo "$1" --version &> /dev/null; then
    echo "Error: cargo $1 is not installed. Please install it with 'rustup component add $1'."
    exit 1
  fi
}

# Check for core components
check_command rustup
check_command cargo
check_cargo_component clippy
check_cargo_component fmt

echo "✅ All required Rust toolchain components are installed."