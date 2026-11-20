# nuc sense review-binding reads a hand-off outside the repository

Observed: `nuc sense review-binding --hand-off <file>` refuses a hand-off that resolves outside
the repository with `[sense/review-binding:file]`, before it tests anything there. The template's
workflow `.github/workflows/review-binding.yml` writes the pull request's description to
`$RUNNER_TEMP/hand-off.md` and passes that path, so the call as the template makes it exits 2
under `nuc`. The bash reads the file wherever it lies.

Two accepted ADRs meet here. API-git-only-sensors-under-sense has `nuc sense` take the arguments
the bash sensor takes, with the same meaning. API-measure-reads-harness-transcripts holds every
read outside the repository to `nuc measure`, and the structural test of
`tests/functional_core.rs` holds that. Both cannot hold for this call. The refusal keeps the second
until the maintainer decides.

The extension: an ADR that lets `nuc sense review-binding` read the one file `--hand-off` names
wherever it lies, read-only, as a regular file or a link to one. The shell function that tests and
reads it joins those `tests/functional_core.guarded` names. A workaround needs no change of `nuc`,
but one of the workflow: it writes the description inside the working tree, or under `.git/` of a
plain checkout, and passes that path. In a linked worktree or a submodule `.git` is a file, and
the git directory lies outside the working tree.

Where: `src/shell/sense.rs` (`review_binding`, `ReviewRepo::hand_off`), the contract test
`review_binding_refuses_a_hand_off_outside_the_repository` in `tests/contract.rs`,
`tests/functional_core.guarded`, and the ADR whose clause would change.
