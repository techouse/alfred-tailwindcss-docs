#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <workflow-binary>" >&2
  exit 2
fi

workflow_binary="$1"
if [[ ! -f "$workflow_binary" ]]; then
  echo "Workflow binary does not exist: $workflow_binary" >&2
  exit 1
fi
if [[ ! -x "$workflow_binary" ]]; then
  echo "Workflow binary is not executable: $workflow_binary" >&2
  exit 1
fi

readonly SOURCE_BINARY="$(cd "$(dirname "$workflow_binary")" && pwd)/$(basename "$workflow_binary")"
readonly ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly DIST_DIR="$ROOT_DIR/build/dist"
readonly REPOSITORY_URL="https://github.com/techouse/alfred-tailwindcss-docs"

cd "$ROOT_DIR"

if ! command -v cargo-about >/dev/null 2>&1; then
  echo "cargo-about is required; install it with: cargo install cargo-about --locked --features cli" >&2
  exit 1
fi

./scripts/version-check.sh

readonly VERSION="$(awk '
  /^\[package\]$/ { in_package = 1; next }
  in_package && /^\[/ { exit }
  in_package && /^version = / { gsub(/["[:space:]]/, "", $3); print $3; exit }
' Cargo.toml)"

if [[ -z "$VERSION" ]]; then
  echo "Could not read the package version from Cargo.toml" >&2
  exit 1
fi

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"
cp "$SOURCE_BINARY" "$DIST_DIR/workflow"
cp info.plist LICENSE README.md assets/*.png "$DIST_DIR/"

/usr/libexec/PlistBuddy -c "Set :version $VERSION" "$DIST_DIR/info.plist"
/usr/libexec/PlistBuddy -c "Set :webaddress $REPOSITORY_URL" "$DIST_DIR/info.plist"

cargo-about generate \
  --locked \
  --fail \
  --output-file "$DIST_DIR/THIRD_PARTY_LICENSES.html" \
  about.hbs
