# shellcheck shell=bash
# The files of another repository this tree carries unchanged, each pinned by its blob
# (NUC-hooks-run-nuc §2): the git hooks under .githooks/ that the blob list .githooks/nuc.blobs
# names. scripts/check-hooks.sh holds each to its blob. Sourced by scripts/check-hooks.sh,
# scripts/check-docs.sh, and the map script of the normalize skill, which read no pinned file and
# rewrite none: a reference in one is the other repository's, and a change to one breaks its pin.

# The five hooks a blob list may pin, and the form of the blob of a line '<blob> <hook>': a git
# object name of SHA-1 or of SHA-256.
PINNED_HOOKS=(merge-adr-index pre-commit pre-merge-commit pre-push reference-transaction)
PINNED_BLOB_RE='^[0-9a-f]{40}([0-9a-f]{24})?$'

# pinned_hook <name> — whether the name is one of the five hooks.
pinned_hook() {
  local h
  for h in "${PINNED_HOOKS[@]}"; do [[ "$1" == "$h" ]] && return 0; done
  return 1
}

# pinned_files <root> — the path of each pinned file relative to the root, one a line, in the
# order of the list; nothing without a list. A line '<blob> <hook>' of one of the five hooks names
# one; any other line names none.
pinned_files() {
  local blob hook rest
  [[ -f "$1/.githooks/nuc.blobs" ]] || return 0
  while read -r blob hook rest || [[ -n "$blob" ]]; do
    hook="${hook%$'\r'}"
    [[ "$blob" =~ $PINNED_BLOB_RE && -z "$rest" ]] && pinned_hook "$hook" && echo ".githooks/$hook"
  done < "$1/.githooks/nuc.blobs"
}
