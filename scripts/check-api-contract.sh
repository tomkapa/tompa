#!/usr/bin/env bash
# check-api-contract.sh — Validates that the committed OpenAPI spec and generated
# TypeScript client are in sync with the Rust backend.
#
# Usage:
#   ./scripts/check-api-contract.sh
#
# Exit codes:
#   0 — spec and generated client are up to date
#   1 — drift detected (regenerate with `bun run generate-api`)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$REPO_ROOT/backend"
FRONTEND_DIR="$REPO_ROOT/frontend"
SPEC_FILE="$FRONTEND_DIR/src/api/openapi.json"
GENERATED_DIR="$FRONTEND_DIR/src/api/generated"

echo "── Generating OpenAPI spec from Rust backend ──"
cargo run \
  --manifest-path "$BACKEND_DIR/server/Cargo.toml" \
  --bin generate-openapi \
  2>/dev/null \
  > "$SPEC_FILE.tmp"

echo "── Checking spec for drift ──"
if ! diff -q "$SPEC_FILE" "$SPEC_FILE.tmp" > /dev/null 2>&1; then
  echo ""
  echo "❌  openapi.json is out of date."
  echo "    Run: bun run generate-api"
  mv "$SPEC_FILE.tmp" "$SPEC_FILE"
  exit 1
fi
rm "$SPEC_FILE.tmp"

echo "── Re-generating TypeScript client ──"
TMP_GENERATED="$(mktemp -d)"
# Temporarily override output to a temp dir for comparison
node "$FRONTEND_DIR/node_modules/.bin/orval" \
  --input "$SPEC_FILE" \
  --output "$TMP_GENERATED/client.ts" \
  2>/dev/null

echo "── Checking generated client for drift ──"
if ! diff -rq "$GENERATED_DIR" "$TMP_GENERATED" > /dev/null 2>&1; then
  echo ""
  echo "❌  Generated TypeScript client is out of date."
  echo "    Run: bun run generate-api"
  rm -rf "$TMP_GENERATED"
  exit 1
fi

rm -rf "$TMP_GENERATED"
echo ""
echo "✅  API contract is up to date."
