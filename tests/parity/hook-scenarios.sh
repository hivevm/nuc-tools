# shellcheck shell=bash
# The scenarios of the hooks (ADR-0022), sourced by scripts/lib/hook-parity.sh. Each restates the
# setup of the template's scripts/test-git-hooks.sh for the cases it names, '|' between them,
# and runs one git command:
#     hook_scenario [--bare] <name> <hook> <cases> <setup function> <git args...>
# The setup gets the fixture, one commit on main with a bare remote 'origin', and runs its git
# commands without hooks through hp_git; hp_checks and hp_docs give it stand-in checks.

# shellcheck disable=SC2329 # each setup runs through hp_run
on_branch() { hp_git "$1" switch -q -c feature; }
red_on_branch() { on_branch "$1"; hp_checks "$1" 1; }
unconfigured() { on_branch "$1"; hp_git "$1" config --unset merge.ff; hp_git "$1" config --unset pull.ff; }
detached() { hp_git "$1" checkout -q --detach; }

# merging <dir> <docs exit code> <checks exit code> — stand-in checks as told, a branch 'feature'
# one commit ahead of main that a fast-forward could take, and 'feat/x' at main checked out.
merging() {
  hp_docs "$1" "$2"
  hp_checks "$1" "$3"
  hp_git "$1" switch -q -c feature
  echo feature > "$1/f"
  hp_git "$1" add f
  hp_git "$1" commit -q -m feature
  hp_git "$1" switch -q -c feat/x main
}
merge_red() { merging "$1" 1 0; }
merge_suite() { merging "$1" 0 1; }
merge_green() { merging "$1" 0 0; }
merge_main() { merging "$1" 0 0; hp_git "$1" switch -q main; }

# conflict <dir> <checks exit code> <branch> — a merge into the branch stopped on a conflict,
# resolved and staged, with a green documentation check.
conflict() {
  hp_docs "$1" 0
  hp_checks "$1" "$2"
  hp_git "$1" switch -q -c feature
  echo feature > "$1/f"
  hp_git "$1" add f
  hp_git "$1" commit -q -m feature
  hp_git "$1" switch -q -c feat/x main
  hp_git "$1" switch -q "$3"
  echo other > "$1/f"
  hp_git "$1" add f
  hp_git "$1" commit -q -m other
  hp_git "$1" merge -q -m m feature
  echo resolved > "$1/f"
  hp_git "$1" add f
}
conflict_red() { conflict "$1" 1 feat/x; }
conflict_green() { conflict "$1" 0 feat/x; }
conflict_main() { conflict "$1" 0 main; }

hook_scenario commit-on-main pre-commit "a commit on main" : commit -q --allow-empty -m x
hook_scenario --bare commit-on-main-bare pre-commit "a commit on main" : commit -q --allow-empty -m x
hook_scenario commit-on-branch pre-commit "a commit on a branch" on_branch commit -q --allow-empty -m x
hook_scenario commit-red-checks pre-commit "a commit is not gated by the checks" \
  red_on_branch commit -q --allow-empty -m x
hook_scenario commit-unconfigured pre-commit "a commit in a clone without the merge settings" \
  unconfigured commit -q --allow-empty -m x
hook_scenario commit-detached pre-commit "a commit on a detached HEAD is not on main" \
  detached commit -q --allow-empty -m x
hook_scenario conflict-red pre-commit \
  "concluding a conflicted merge while the checks are red|concluding a conflicted merge runs the checks with --changed" \
  conflict_red commit -q --no-edit
hook_scenario conflict-green pre-commit "concluding a conflicted merge while the checks are green" \
  conflict_green commit -q --no-edit
hook_scenario conflict-main pre-commit "concluding a conflicted merge on main, though the checks are green" \
  conflict_main commit -q --no-edit
hook_scenario merge-red pre-merge-commit \
  "a merge into a branch while the documentation check is red|the refusal shows the documentation check's output" \
  merge_red merge -q -m m feature
hook_scenario merge-suite pre-merge-commit "a clean merge is not gated by the rest of the checks" \
  merge_suite merge -q -m m feature
hook_scenario merge-ff pre-merge-commit "merge.ff false keeps a fast-forward from passing the hook" \
  merge_red merge -q feature
hook_scenario merge-green pre-merge-commit "a merge into a branch while the checks are green" \
  merge_green merge -q -m m feature
hook_scenario merge-main pre-merge-commit "a merge on main, though the checks are green" \
  merge_main merge -q -m m feature
hook_scenario --bare merge-main-bare pre-merge-commit "a merge on main, though the checks are green" \
  merge_main merge -q -m m feature
