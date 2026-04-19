#!/usr/bin/env bash
set -euo pipefail

# Push a release tag in v* format to trigger GitHub Actions release/publish flows.
# Workflow policy:
# 1) Require clean working tree.
# 2) Push current branch to origin first.
# 3) Tag the latest remote commit (origin/<branch>), not local-only state.
# Usage:
#   ./scripts/push-tag.sh                       # auto bump patch from latest v* tag, tag HEAD
#   ./scripts/push-tag.sh <commit-ish>         # auto bump patch, tag given commit
#   ./scripts/push-tag.sh 0.0.10
#   ./scripts/push-tag.sh v0.0.10
#   ./scripts/push-tag.sh v0.0.10 <commit-ish>

if [[ $# -gt 2 ]]; then
  echo "Usage: $0 [version|vX.Y.Z|commit-ish] [commit-ish]"
  exit 1
fi

auto_next_tag() {
  local latest major minor patch
  latest="$(git tag --list 'v[0-9]*.[0-9]*.[0-9]*' --sort=-v:refname | head -n1 || true)"
  if [[ -z "$latest" ]]; then
    echo "v0.0.1"
    return
  fi
  latest="${latest#v}"
  IFS='.' read -r major minor patch <<<"$latest"
  patch=$((patch + 1))
  echo "v${major}.${minor}.${patch}"
}

read_cargo_version() {
  awk -F'"' '/^version = "/{print $2; exit}' Cargo.toml
}

read_npm_version() {
  if command -v node >/dev/null 2>&1; then
    node -p "require('./package.json').version"
    return
  fi
  awk -F'"' '/"version"\s*:\s*"/{print $4; exit}' package.json
}

arg1="${1:-}"
arg2="${2:-}"
target_ref=""
branch="$(git branch --show-current)"

if [[ -z "$branch" ]]; then
  echo "Detached HEAD is not supported for this release flow."
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree is not clean. Commit or stash changes before tagging."
  exit 1
fi

echo "Pushing current branch '${branch}' to origin..."
git push origin "$branch"
git fetch --tags origin

if [[ -z "$arg1" ]]; then
  tag="$(auto_next_tag)"
  target_ref="origin/${branch}"
elif [[ "$arg1" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  tag="${arg1#v}"
  tag="v${tag}"
  target_ref="${arg2:-origin/${branch}}"
else
  # First arg is treated as commit-ish, version is auto-incremented.
  tag="$(auto_next_tag)"
  target_ref="$arg1"
fi

if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid tag format: $tag"
  echo "Expected: vMAJOR.MINOR.PATCH (example: v0.0.10)"
  exit 1
fi

tag_version="${tag#v}"
cargo_version="$(read_cargo_version)"
npm_version="$(read_npm_version)"

if [[ "$cargo_version" != "$tag_version" || "$npm_version" != "$tag_version" ]]; then
  echo "Version mismatch; refusing to tag."
  echo "Tag version:   $tag_version"
  echo "Cargo.toml:    $cargo_version"
  echo "package.json:  $npm_version"
  echo "Update versions first, commit, then run this script again."
  exit 1
fi

if ! git rev-parse --verify "$target_ref" >/dev/null 2>&1; then
  echo "Invalid commit-ish: $target_ref"
  exit 1
fi

if git rev-parse --verify "$tag" >/dev/null 2>&1; then
  echo "Tag already exists locally: $tag"
  exit 1
fi

echo "Creating tag ${tag} at ${target_ref}..."
git tag "$tag" "$target_ref"

echo "Pushing tag ${tag} to origin..."
git push origin "$tag"

echo "Done. GitHub workflows listening on tags 'v*' should now start."
