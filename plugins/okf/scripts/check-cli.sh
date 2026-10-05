#!/bin/sh
# Read-only preflight. Never downloads or installs software.
set -eu
plugin_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
read -r minimum maximum < "$plugin_root/cli-compatibility.txt"
help_install() {
  echo "Required CLI: >=$minimum, <$maximum (stable releases)." >&2
  echo 'Installation: https://github.com/abatyuk/okf#install-a-release-no-rust-required' >&2
  echo 'Download a matching GitHub release or manually run the optional standalone installer. Skills must not invoke it.' >&2
}
resolved=$(command -v okf || true)
if [ -z "$resolved" ]; then echo 'okf is not on PATH.' >&2; help_install; exit 1; fi
if ! reported=$(okf version 2>/dev/null); then
  echo "Cannot run okf at $resolved." >&2; help_install; exit 1
fi
version=$(printf '%s\n' "$reported" | awk 'NR == 1 && $1 == "okf" {print $2}')
if ! awk -v version="$version" -v minimum="$minimum" -v maximum="$maximum" '
function valid(v) {return v ~ /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/}
function compare(a,b, aa,bb,i) {
 split(a,aa,"."); split(b,bb,".");
 for(i=1;i<=3;i++) {if(aa[i]+0<bb[i]+0)return -1; if(aa[i]+0>bb[i]+0)return 1} return 0
}
BEGIN {exit !(valid(version) && valid(minimum) && valid(maximum) && compare(version,minimum)>=0 && compare(version,maximum)<0)}'; then
  echo "Incompatible CLI at $resolved: $reported" >&2; help_install; exit 1
fi
echo "Compatible CLI: $resolved ($version; required >=$minimum, <$maximum)."
