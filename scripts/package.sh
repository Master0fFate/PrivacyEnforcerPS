#!/usr/bin/env bash
# Packages built executables into dist/ with checksums and build metadata.
#
# Usage: scripts/package.sh [TAG]
#   TAG defaults to v<workspace version>. Builds are not started here; run cargo first:
#     cargo build --release --workspace
#     cargo build --release -p pef-cli --target x86_64-pc-windows-gnu
#     cargo build --profile cross -p pef-gui --target x86_64-pc-windows-gnu
set -euo pipefail
cd "$(dirname "$0")/.."

version="$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
tag="${1:-v$version}"
commit="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
toolchain="$(rustc --version)"
rm -rf dist
mkdir -p dist

package() {
  local target="$1" bin_dir="$2" ext="$3" archive_kind="$4" note="$5"
  local name="privacy-enforcer-${tag}-${target}"
  local dir="dist/$name"
  [ -f "$bin_dir/pef$ext" ] || { echo "skip $target: $bin_dir/pef$ext missing"; return; }
  mkdir -p "$dir"
  cp "$bin_dir/pef$ext" "$dir/"
  if [ -f "$bin_dir/pef-gui$ext" ]; then cp "$bin_dir/pef-gui$ext" "$dir/"; fi
  cp README.md LICENSE CHANGELOG.md "$dir/"
  cp crates/pef-gui/assets/fonts/OFL.txt "$dir/FONT-LICENSE-OFL.txt"
  if [ -n "$note" ]; then printf '%s\n' "$note" > "$dir/BUILD-NOTE.txt"; fi
  local archive
  if [ "$archive_kind" = zip ]; then
    archive="$name.zip"
    (cd dist && zip -qr "$archive" "$name")
  else
    archive="$name.tar.gz"
    (cd dist && tar czf "$archive" "$name")
  fi
  local sha
  sha="$(cd dist && sha256sum "$archive" | cut -d' ' -f1)"
  cat > "dist/$name.build-metadata.json" <<JSON
{
  "sourceCommit": "$commit",
  "tag": "$tag",
  "target": "$target",
  "toolchain": "$toolchain",
  "signed": false,
  "archive": "$archive",
  "sha256": "$sha"
}
JSON
  echo "packaged dist/$archive"
}

package x86_64-unknown-linux-gnu target/release "" tar.gz ""
# Windows: the CLI comes from the release profile, the desktop app from the cross profile.
win_dir="dist/.win-staging"
rm -rf "$win_dir"; mkdir -p "$win_dir"
[ -f target/x86_64-pc-windows-gnu/release/pef.exe ] && cp target/x86_64-pc-windows-gnu/release/pef.exe "$win_dir/"
[ -f target/x86_64-pc-windows-gnu/cross/pef-gui.exe ] && cp target/x86_64-pc-windows-gnu/cross/pef-gui.exe "$win_dir/"
if [ -f "$win_dir/pef.exe" ]; then
  package x86_64-pc-windows-gnu "$win_dir" ".exe" zip \
"These Windows executables were cross-built from Linux with the MinGW-w64 toolchain
(target x86_64-pc-windows-gnu). pef-gui.exe uses the 'cross' profile: release
optimisations with debug assertions on, so the renderer compiles its shaders at
first start instead of embedding precompiled ones. Releases built natively on
Windows by the GitHub Actions workflow use the plain release profile. Binaries are
unsigned; compare the SHA-256 in the matching .build-metadata.json before running."
fi
rm -rf "$win_dir"

(cd dist && sha256sum *.zip *.tar.gz 2>/dev/null > SHA256SUMS.txt || true)
echo
cat dist/SHA256SUMS.txt
