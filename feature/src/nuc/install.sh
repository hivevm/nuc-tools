#!/bin/sh
#
# The Dev Container Feature's installer (ADR-0014): installs nuc into
# /usr/local/bin from the release this Feature version carries. The release workflow's
# release/pin.sh puts install-nuc.sh, SHA256SUMS, and VERSION beside this file; without them, as
# on main, the installation fails.

set -eu
here="$(cd "$(dirname "$0")" && pwd)"
[ -f "$here/install-nuc.sh" ] \
  || { echo "nuc Feature: this Feature carries no release; use a published version" >&2; exit 1; }
exec sh "$here/install-nuc.sh" "$here/SHA256SUMS" "$here/VERSION" /usr/local/bin
