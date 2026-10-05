#!/bin/sh
# Standalone installer: no Rust, Python, GitHub CLI, or existing okf required.
set -eu
fail() { echo "okf installer: $*" >&2; exit 1; }
usage() { echo 'Usage: sh install.sh VERSION [INSTALL_DIRECTORY]'; echo 'Example: sh install.sh 0.3.2 "$HOME/.local/bin"'; }
[ "${1:-}" != '--help' ] || { usage; exit 0; }
[ "$#" -ge 1 ] && [ "$#" -le 2 ] || { usage >&2; exit 2; }
version=${1#v}
printf '%s\n' "$version" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' || fail 'Use a stable version such as 0.3.2.'
destination=${2:-"$HOME/.local/bin"}
case "$destination" in /*) ;; *) destination="$PWD/$destination" ;; esac
case "$(uname -s)/$(uname -m)" in
  Darwin/arm64|Darwin/aarch64) target=aarch64-apple-darwin ;;
  Darwin/x86_64) target=x86_64-apple-darwin ;;
  Linux/aarch64|Linux/arm64) target=aarch64-unknown-linux-musl ;;
  Linux/x86_64) target=x86_64-unknown-linux-musl ;;
  *) fail 'Supported platforms: macOS/Linux, x86_64/ARM64.' ;;
esac
command -v curl >/dev/null 2>&1 || fail 'curl is required.'
command -v tar >/dev/null 2>&1 || fail 'tar is required.'
if command -v sha256sum >/dev/null 2>&1; then checksum=sha256sum
elif command -v shasum >/dev/null 2>&1; then checksum=shasum
else fail 'sha256sum or shasum is required.'; fi
work=$(mktemp -d)
staged=
trap 'rm -rf "$work"; if [ -n "$staged" ]; then rm -f "$staged"; fi' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
asset="okf-$version-$target.tar.gz"
base="https://github.com/abatyuk/okf/releases/download/v$version"
for name in "$asset" SHA256SUMS; do
  curl --fail --location --silent --show-error --proto '=https' --proto-redir '=https' \
    "$base/$name" -o "$work/$name"
done
expected=$(awk -v name="$asset" '$2 == name {print $1}' "$work/SHA256SUMS")
printf '%s\n' "$expected" | grep -Eq '^[0-9a-f]{64}$' || fail 'Missing or ambiguous archive checksum.'
if [ "$checksum" = sha256sum ]; then actual=$(sha256sum "$work/$asset" | awk '{print $1}')
else actual=$(shasum -a 256 "$work/$asset" | awk '{print $1}'); fi
[ "$actual" = "$expected" ] || fail 'Checksum mismatch; nothing installed.'
# Extract only the executable, and reject a symlink or directory.
tar -xzf "$work/$asset" -C "$work" okf
[ -f "$work/okf" ] && [ ! -L "$work/okf" ] || fail 'Archive does not contain a regular okf executable.'
chmod 755 "$work/okf"
reported=$("$work/okf" version) || fail 'Downloaded binary cannot run on this system.'
[ "$reported" = "okf $version (OKF spec 0.2)" ] || fail "Unexpected binary version: $reported"
mkdir -p "$destination"
[ ! -L "$destination/okf" ] || fail "Refusing to replace symlink $destination/okf; use its package manager."
[ ! -d "$destination/okf" ] || fail "Destination is a directory: $destination/okf"
staged=$(mktemp "$destination/.okf-install.XXXXXX")
cp "$work/okf" "$staged"
chmod 755 "$staged"
mv -f "$staged" "$destination/okf"
staged=
echo "Installed okf $version at $destination/okf"
resolved=$(command -v okf || true)
if [ "$resolved" != "$destination/okf" ]; then
  echo "Your shell currently resolves okf to: ${resolved:-not found}"
  echo "Put $destination before other okf installations on PATH, then restart your shell."
fi
