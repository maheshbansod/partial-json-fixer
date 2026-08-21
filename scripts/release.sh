#!/usr/bin/env bash
# Cut a release: bump versions, commit, tag, push.
#
#   ./scripts/release.sh patch   # 0.5.3 -> 0.5.4
#   ./scripts/release.sh minor   # 0.5.3 -> 0.6.0
#
# Pushing the v* tag triggers CI to build wheels and publish to PyPI,
# plus publish the core Rust crate to crates.io.
set -euo pipefail

cd "$(dirname "$0")/.."

BUMP="${1:-}"
if [[ ! "$BUMP" =~ ^(patch|minor|major)$ ]]; then
  echo "usage: ./scripts/release.sh <patch|minor|major>" >&2
  exit 1
fi

CORE="crates/partial-json-fixer/Cargo.toml"
PY="py/partial-json-fixer/Cargo.toml"

OLD=$(sed -n 's/^version = "\(.*\)"/\1/p' "$CORE" | head -1)

IFS=. read -r MAJ MIN PAT <<<"$OLD"
case "$BUMP" in
  major) MAJ=$((MAJ + 1)) ;;
  minor) MIN=$((MIN + 1)) ;;
  patch) PAT=$((PAT + 1)) ;;
esac
NEW="$MAJ.$MIN.$PAT"

if git rev-parse "v$NEW" >/dev/null 2>&1; then
  echo "error: tag v$NEW already exists" >&2
  exit 1
fi

echo "bumping $OLD -> $NEW"

for f in "$CORE" "$PY"; do
  sed -i '' 's/^version = ".*"/version = "'"$NEW"'"/' "$f"
done

cargo update --manifest-path crates/partial-json-fixer/Cargo.toml -p partial-json-fixer >/dev/null
cargo update --manifest-path py/partial-json-fixer/Cargo.toml -p partial-json-fixer-py >/dev/null

BRANCH=$(git branch --show-current)
if [[ "$BRANCH" != "main" && "$BRANCH" != "master" ]]; then
  echo "error: run this from main (currently on $BRANCH)" >&2
  exit 1
fi

git add "$CORE" "$PY" crates/partial-json-fixer/Cargo.lock py/partial-json-fixer/Cargo.lock
git commit -m "release v$NEW"
git tag "v$NEW"

echo
echo "ready: commit + tag v$NEW created locally."
read -rp "push to origin now? [y/N] " answer
if [[ "$answer" =~ ^[Yy]$ ]]; then
  git push origin "$BRANCH" "v$NEW"
  echo "tag pushed — CI will publish to PyPI and crates.io:"
  echo "  https://github.com/$(git remote get-url origin | sed -E 's#.*(github.com[:/])##; s/\.git$//')/actions"
else
  echo "not pushed. when ready: git push origin $BRANCH v$NEW"
fi
