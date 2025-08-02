#!/bin/sh
# Copyright 2025 Cowboy AI, LLC.

# Run cargo check in Nix environment
echo "Running cargo check in Nix environment..."
nix develop -c cargo check 2>&1