#!/usr/bin/env bash
#
# Self-test of .devcontainer/gh-host-login.sh, the gh the Dev Container installs.
#
# Each case runs a copy of the wrapper that calls a stub in place of the real gh, under a stub git
# that answers as a credential helper would. No case reads a real login or runs the real gh. The
# forwarding by the Dev Containers extension itself is not tested: it exists only in a window
# connected to a container.
#
# Inputs: .devcontainer/gh-host-login.sh .devcontainer/devcontainer.json scripts/lib/
#
# Usage:
#     scripts/test-gh-host-login.sh
# Exit code 0 when every case passes, 1 otherwise.

# shellcheck disable=SC2317 # the assertions run through check, which ShellCheck cannot follow
set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"

WRAPPER="$ROOT/.devcontainer/gh-host-login.sh"
CONFIG="$ROOT/.devcontainer/devcontainer.json"
export STUB_LOG="$work/git-calls"

mkdir -p "$work/bin"
sed "s#/usr/bin/gh#$work/real-gh#" "$WRAPPER" > "$work/bin/gh"
chmod +x "$work/bin/gh"

# The real gh's stand-in: reports the token it sees and its arguments, and exits as told.
cat > "$work/real-gh" <<'EOF'
#!/bin/sh
echo "token=${GH_TOKEN-<unset>} args=$*"
exit "${STUB_GH_EXIT:-0}"
EOF
chmod +x "$work/real-gh"

# git's stand-in: logs the call and the prompt settings, and answers as told. STUB_GIT_RECURSE
# makes it call gh, as a credential helper 'gh auth git-credential' would; the log's length stops
# a wrapper without its guard.
cat > "$work/bin/git" <<'EOF'
#!/bin/sh
echo "$* prompt=${GIT_TERMINAL_PROMPT-<unset>} askpass=${GIT_ASKPASS-<unset>}" >> "$STUB_LOG"
if [ -n "${STUB_GIT_RECURSE:-}" ] && [ "$(wc -l < "$STUB_LOG")" -lt 5 ]; then
  gh auth git-credential >/dev/null
fi
cat >/dev/null
[ -z "${STUB_GIT_FAIL:-}" ] || exit 1
printf 'protocol=https\nhost=github.com\nusername=someone\n'
[ -z "${STUB_PASSWORD:-}" ] || printf 'password=%s\n' "$STUB_PASSWORD"
EOF
chmod +x "$work/bin/git"

# run_gh [VAR=value...] -- <gh arguments...> — run the wrapper from an empty log; sets output and
# status.
run_gh() {
  local vars=()
  while [[ "$1" != "--" ]]; do vars+=("$1"); shift; done
  shift
  : > "$STUB_LOG"
  output="$(env -u GH_TOKEN -u GH_HOST_LOGIN_ASKED PATH="$work/bin:$PATH" "${vars[@]}" "$work/bin/gh" "$@" 2>&1)"
  status=$?
}
output_is() { [[ "$output" == "$1" ]]; }
status_is() { [[ "$status" == "$1" ]]; }
calls_are() { [[ "$(wc -l < "$STUB_LOG" | tr -d ' ')" == "$1" ]]; }
log_has() { grep -qF -- "$1" "$STUB_LOG"; }

run_gh STUB_PASSWORD=from-host -- pr view 'a b'
check "the host's login reaches gh, with its arguments" output_is "token=from-host args=pr view a b"
check "git is asked once" calls_are 1
check "git is asked for the credential" log_has "credential fill"
check "git has no terminal to prompt on" log_has "prompt=0"
askpass_empty() { grep -qE 'askpass=$' "$STUB_LOG"; }
check "git has no askpass to prompt with" askpass_empty

run_gh -- auth status
check "no login leaves GH_TOKEN unset" output_is "token=<unset> args=auth status"

run_gh STUB_GIT_FAIL=1 STUB_PASSWORD=ignored -- auth status
check "a failing git leaves GH_TOKEN unset" output_is "token=<unset> args=auth status"
check "a failing git does not stop gh" status_is 0

run_gh GH_TOKEN=by-hand STUB_PASSWORD=from-host -- api user
check "a token set by hand wins" output_is "token=by-hand args=api user"
check "a token set by hand asks git nothing" calls_are 0

run_gh STUB_GH_EXIT=7 STUB_PASSWORD=from-host -- api user
check "gh's exit status comes through" status_is 7

run_gh STUB_GIT_RECURSE=1 STUB_PASSWORD=from-host -- api user
check "gh as git's helper does not recurse" calls_are 1
check "gh as git's helper still gets the login" output_is "token=from-host args=api user"

installs() { grep -qF 'sudo install -m 0755 .devcontainer/gh-host-login.sh /usr/local/bin/gh' "$CONFIG"; }
check "the container installs the wrapper as gh" installs

summary "gh host login self-test"
