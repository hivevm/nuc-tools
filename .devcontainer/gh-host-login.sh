#!/bin/sh
#
# The GitHub CLI with the host's login. The container installs this file as /usr/local/bin/gh, ahead
# of the real one on PATH (postCreateCommand of devcontainer.json).
#
# gh does not ask a git credential helper, and the Dev Containers extension forwards only git's
# requests to the host. So each call asks git for the login of github.com and hands it to this one
# process, and no file or container-wide variable holds it. CONTRIBUTING.md -> "GitHub access".
#
# A token set by hand wins. Without an answer, gh runs with what it has of its own, and a login made
# with 'gh auth login' in the container still works. Nothing prompts: this runs under commands that
# have no terminal. The guard stops the recursion where git's helper is this gh itself, as a
# 'gh auth git-credential' found on PATH would be.
#
# Self-test: scripts/test-gh-host-login.sh

if [ -z "${GH_TOKEN:-}" ] && [ -z "${GH_HOST_LOGIN_ASKED:-}" ]; then
  GH_HOST_LOGIN_ASKED=1
  export GH_HOST_LOGIN_ASKED
  token="$(printf 'protocol=https\nhost=github.com\n\n' \
    | GIT_TERMINAL_PROMPT=0 GIT_ASKPASS='' git credential fill 2>/dev/null \
    | sed -n 's/^password=//p')"
  if [ -n "$token" ]; then
    GH_TOKEN="$token"
    export GH_TOKEN
  fi
fi

exec /usr/bin/gh "$@"
