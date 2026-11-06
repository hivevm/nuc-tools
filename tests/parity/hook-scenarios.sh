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

# delete_red <dir> — 'feature' and 'other' on the remote besides main, and red checks.
delete_red() { on_branch "$1"; hp_git "$1" push -q origin main feature feature:other; hp_checks "$1" 1; }
# deleted_red <dir> — as delete_red after 'feature' was deleted from the remote, so that a push
# of it sends it again.
deleted_red() { delete_red "$1"; hp_git "$1" push -q origin --delete feature; }

# drafts <dir> — a branch whose commit carries a draft ADR, an ADR, and a draft that is no
# Markdown, with green checks.
drafts() {
  on_branch "$1"
  hp_checks "$1" 0
  mkdir -p "$1/docs/adr"
  : > "$1/docs/adr/draft-x.md"
  : > "$1/docs/adr/0001-y.md"
  : > "$1/docs/adr/draft-notes.txt"
  hp_git "$1" add docs
  hp_git "$1" commit -q -m draft
}

# green_tree <dir> <verdict> — a branch whose committed check script exits as .git/verdict says,
# read from outside the tree, so that the same tree can turn red; the verdict as given.
green_tree() {
  on_branch "$1"
  mkdir -p "$1/scripts"
  # shellcheck disable=SC2016 # the stand-in's own expansion
  printf '#!/usr/bin/env bash\necho "check output for the test"\nexit "$(cat .git/verdict)"\n' \
    > "$1/scripts/check-all.sh"
  hp_git "$1" add scripts
  hp_git "$1" commit -q -m checks
  echo "$2" > "$1/.git/verdict"
}
# recorded <dir> — a green tree written into the record by hand, the checks turned red, and the
# branch reworked by an amend that keeps the tree.
recorded() {
  green_tree "$1" 1
  hp_git "$1" rev-parse 'HEAD^{tree}' > "$1/.git/checks-green-trees"
  hp_git "$1" commit -q --amend -m regrouped
}
green_clean() { green_tree "$1" 0; }
recorded_untracked() { recorded "$1"; touch "$1/untracked"; }
recorded_new() { recorded "$1"; echo x > "$1/new"; hp_git "$1" add new; hp_git "$1" commit -q -m new; }

# linked <dir> — a linked worktree 'wt' on 'feature' inside the fixture, whose check script fails
# on any variable that binds git to a repository.
linked() {
  hp_git "$1" worktree add -q "$1/wt" -b feature
  mkdir -p "$1/wt/scripts"
  # shellcheck disable=SC2016 # the stand-in's own expansion
  printf '%s\n' '#!/usr/bin/env bash' \
    'for v in $(git rev-parse --local-env-vars); do' \
    '  [[ -v $v ]] && { echo "check output for the test: leaked $v"; exit 1; }' \
    'done' \
    'echo "check output for the test: no git environment"' > "$1/wt/scripts/check-all.sh"
}

hook_scenario push-to-main pre-push "a push to the remote's main" : push -q origin main
hook_scenario --bare push-to-main-bare pre-push "a push to the remote's main" : push -q origin main
hook_scenario push-main-other pre-push "a push of main to a branch of its own with no check script to run" \
  : push -q origin main:other
hook_scenario push-red pre-push \
  "a push while the checks are red|the refusal shows the check output|the hook runs the checks with --changed" \
  red_on_branch push -q origin feature
hook_scenario delete-branch pre-push "a deletion of a branch while the checks are red" \
  delete_red push -q origin --delete feature
hook_scenario delete-then-push pre-push "a push of a branch while the checks are red, after a deletion passed" \
  deleted_red push -q origin feature
hook_scenario delete-and-push pre-push "a deletion together with a push of a branch is checked" \
  deleted_red push -q origin :other feature
hook_scenario delete-main pre-push \
  "a deletion of the remote's main while the checks are red|a deletion of the remote's main" \
  delete_red push -q origin --delete main
hook_scenario push-green pre-push "a push while the checks are green" \
  merge_green push -q origin feature
hook_scenario push-drafts pre-push "" drafts push -q origin feature
hook_scenario green-clean pre-push "a push of a clean tree while the checks are green" \
  green_clean push -q origin feature
hook_scenario green-reworked pre-push "a reworked branch on a tree that passed before, while the checks are red" \
  recorded push -q --force origin feature
hook_scenario green-untracked pre-push "the same tree with an untracked file is checked again" \
  recorded_untracked push -q --force origin feature
hook_scenario green-new pre-push "a new tree is checked again" recorded_new push -q origin feature
hook_scenario push-linked pre-push "a push from a linked worktree hands the checks no git environment" \
  linked -C wt push -q origin feature
hook_scenario push-unconfigured pre-push "a push from a clone without the merge settings" \
  unconfigured push -q origin feature
