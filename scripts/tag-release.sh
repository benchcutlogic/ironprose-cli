#!/usr/bin/env bash
# Create a git tag for cargo-dist to trigger release
# Called by `pnpm run tag-release` during changeset publish
set -euo pipefail

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$VERSION" ]; then
  echo "Could not determine version from Cargo.toml"
  exit 1
fi

TAG="v$VERSION"

echo "Tagging release: $TAG"
git tag "$TAG"
git push origin "$TAG"
