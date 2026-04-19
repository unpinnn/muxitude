#!/usr/bin/env bash
# Sync local workspace to remote Termux host, clear cache DB, then run release build.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  ./scripts/build-remote.sh --remote <user@host> --source <local_path> --target <remote_sync_path> [options]

Arguments:
  --source       Local path to sync.
  --target       Remote sync target path.
  --build-dir    Remote directory where "cargo build --release" is run (default: --target).
  --remote       SSH destination in user@host format.
  --local-source Deprecated alias for --source.

Example:
  ./scripts/build-remote.sh --remote root@192.168.76.1 --port 65500 --source . --target workspace/muxitude
USAGE
}

remote=""
target=""
source_dir=""
build_dir=""
port="22"
key_path=""
explicit_key=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote|-r)
      remote="${2:-}"
      shift 2
      ;;
    --target|--tagret|-t)
      target="${2:-}"
      shift 2
      ;;
    --source|-s)
      source_dir="${2:-}"
      shift 2
      ;;
    --local-source|-l)
      source_dir="${2:-}"
      shift 2
      ;;
    --build-dir|-b)
      build_dir="${2:-}"
      shift 2
      ;;
    --port|-p)
      port="${2:-}"
      shift 2
      ;;
    --key|-k)
      key_path="${2:-}"
      explicit_key=1
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Error: unknown option '$1'"
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$remote" || -z "$target" || -z "$source_dir" ]]; then
  echo "Error: --remote, --source and --target are required."
  usage
  exit 1
fi
if [[ -z "$build_dir" ]]; then
  build_dir="$target"
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
sync_script="$script_dir/sync-remote.sh"

if [[ ! -f "$sync_script" ]]; then
  echo "Error: sync script not found: $sync_script"
  exit 1
fi

ssh_args=(-p "$port")
sync_args=(--remote "$remote" --port "$port" --source "$source_dir" --target "$target")

if [[ "$explicit_key" -eq 1 ]]; then
  sync_args+=(--key "$key_path")
  ssh_args+=(-i "$key_path")
fi

echo "Step 1/3: Syncing local project to remote..."
"$sync_script" "${sync_args[@]}"

echo "Step 2/3: Clearing remote muxitude cache DB..."
ssh "${ssh_args[@]}" "$remote" 'cache_db="${XDG_CACHE_HOME:-$HOME/.cache}/muxitude/packages.db"; rm -f "$cache_db"'

echo "Step 3/3: Building on remote in '$build_dir'..."
ssh "${ssh_args[@]}" "$remote" "cd \"$build_dir\" && cargo build --release"

echo "Remote build complete."
