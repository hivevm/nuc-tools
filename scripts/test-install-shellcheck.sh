#!/usr/bin/env bash
#
# Self-test of .devcontainer/install-shellcheck.sh, the one ShellCheck of the Dev Container and of
# the shell job of CI.
#
# Each case serves a fake release archive from a directory under $work through
# SHELLCHECK_BASE_URL, so no case reaches the network. A copy of the script whose checksums are
# those of the fake archive installs it; the script itself refuses it. The download of the real
# archive is not tested: it needs the network, and the shell job of CI runs it on every push.
#
# Inputs: .devcontainer/install-shellcheck.sh .devcontainer/devcontainer.json
#         .github/workflows/checks.yml scripts/lib/
#
# Usage:
#     scripts/test-install-shellcheck.sh
# Exit code 0 when every case passes, 1 otherwise.

# shellcheck disable=SC2317,SC2329 # the assertions run through check, which ShellCheck cannot follow
set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"

SCRIPT="$ROOT/.devcontainer/install-shellcheck.sh"
CONFIG="$ROOT/.devcontainer/devcontainer.json"
WORKFLOW="$ROOT/.github/workflows/checks.yml"
version="$(bash "$SCRIPT" --version)"

# A fake release: an archive of the layout the real one has, holding a shellcheck that prints a
# version as the real one does.
case "$(uname -m)" in
  aarch64 | arm64) arch=aarch64 ;;
  *) arch=x86_64 ;;
esac
mkdir -p "$work/release/v$version" "$work/pack/shellcheck-v$version"
cat > "$work/pack/shellcheck-v$version/shellcheck" <<EOF
#!/bin/sh
printf 'ShellCheck - shell script analysis tool\nversion: $version\n'
EOF
chmod +x "$work/pack/shellcheck-v$version/shellcheck"
archive="$work/release/v$version/shellcheck-v$version.linux.$arch.tar.xz"
tar -cJf "$archive" -C "$work/pack" "shellcheck-v$version"
fake_sha="$(sha256sum "$archive")"
fake_sha="${fake_sha%% *}"

# The script with the fake archive's checksums in place of the pinned ones.
sed -E "s/^(SHA256_[a-z0-9_]+)=.*/\1=$fake_sha/" "$SCRIPT" > "$work/trusting.sh"

# run <script> <arguments...> — run it against the fake release; sets output and status.
run() {
  output="$(SHELLCHECK_BASE_URL="file://$work/release" bash "$@" 2>&1)"
  status=$?
}
status_is() { [[ "$status" == "$1" ]]; }
output_has() { [[ "$output" == *"$1"* ]]; }
absent() { [[ ! -e "$1" ]]; }
reports_version() { [[ "$("$1" --version | sed -n 's/^version: //p')" == "$version" ]]; }

run "$SCRIPT" "$work/refused"
check "an archive with another checksum is refused" status_is 1
check "the refusal names the pinned checksum" output_has "not the pinned"
check "a refused archive installs nothing" absent "$work/refused/shellcheck"

run "$work/trusting.sh" "$work/installed"
check "an archive with the pinned checksum installs" status_is 0
check "the installed shellcheck is the pinned version" reports_version "$work/installed/shellcheck"

run "$work/trusting.sh" "$work/none" extra
check "a second directory is refused" status_is 2
run "$SCRIPT"
check "no directory is refused" status_is 2
run "$SCRIPT" --frobnicate
check "an unknown option is refused" status_is 2

output="$(PATH="$work/installed:$PATH" bash "$SCRIPT" --verify 2>&1)"; status=$?
check "the pinned version on PATH verifies" status_is 0
mkdir -p "$work/other"
printf '#!/bin/sh\nprintf "version: 0.0.1\\n"\n' > "$work/other/shellcheck"
chmod +x "$work/other/shellcheck"
output="$(PATH="$work/other:$PATH" bash "$SCRIPT" --verify 2>&1)"; status=$?
check "another version on PATH fails" status_is 1
check "the failure names the version found" output_has "'0.0.1'"

container_installs() { grep -qF 'bash .devcontainer/install-shellcheck.sh /usr/local/bin' "$CONFIG"; }
check "the container installs the pinned shellcheck" container_installs
no_apt() { ! grep -qE 'apt-get install[^"]*shellcheck' "$CONFIG"; }
check "the container installs no shellcheck of the archive" no_apt
# shellcheck disable=SC2016 # the variable is the workflow's, not the shell's
ci_installs() { grep -qF 'bash .devcontainer/install-shellcheck.sh "$RUNNER_TEMP/shellcheck"' "$WORKFLOW"; }
check "the shell job installs the pinned shellcheck" ci_installs
ci_verifies() { grep -qF 'bash .devcontainer/install-shellcheck.sh --verify' "$WORKFLOW"; }
check "the shell job verifies the version it lints with" ci_verifies

summary "ShellCheck install self-test"
