#!/usr/bin/env bash
# Sync the version from .changeset into Cargo.toml
# Called by `pnpm run version-sync` during changeset version
set -euo pipefail

# Extract version from package.json (changesets writes it there)
VERSION=$(node -p "require('./package.json').version // '0.1.0'")

# Update Cargo.toml version
perl -i -pe "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

echo "Synced Cargo.toml version to $VERSION"
