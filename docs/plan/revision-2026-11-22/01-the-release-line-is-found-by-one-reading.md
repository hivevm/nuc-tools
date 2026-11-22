# Ticket: The release line is found by one reading

## What to build

`nuc upgrade` reads the release a project carries through `docs_template_release::carried`, and
moves it in `upgrade::move_release_line`. The second function finds the line again on its own:
it takes the first line that holds the label and that `carried` reads a release from, then looks
for the tag after the label past spaces, brackets, and backticks. So the place of the release on
its line is read twice, in two shapes, and the two can disagree on a README neither has a case
for.

The evidence is the review of ticket 02 of `nuc-upgrade`. Its first round found the README read
lossily and written back. Its second round found the fix's regression: a later release line was
written where the first was no UTF-8, because the second reading skipped a line the first had
taken. Both are fixed; the two readings stay. The smell is logic written a second time.

`docs_template_release` gives the byte range of the release on the first Template release line,
the one `carried` reads. `carried` and `move_release_line` both take it, so the line `nuc upgrade`
moves is by construction the line it read the release from. No finding of `docs/template-release`
and no release `nuc` reads changes.

## Acceptance criteria

- [ ] Given a README whose first Template release line names a release, when `nuc upgrade` moves
  it, then exactly the bytes of that tag are replaced, as the unit tests of
  `move_release_line` show today.
- [ ] Given a README whose first release line names no release and a later one names one, or whose
  first is no UTF-8, when `releases` and `move_release_line` read it, then neither reads the later
  line, and a unit test holds each case.
- [ ] Given the change, when `src/upgrade.rs` is read, then it holds no search for the label or
  the tag of its own.
- [ ] Given the parity harness and the fixture tables, when they run, then `docs/template-release`
  reports what it reported before.

## Read first

- Touches: `src/rules/docs_template_release.rs`, `src/upgrade.rs`
- API-upgrade-by-blob-toml-optional (*The release line*); API-version-coupling-hook-defaults; the
  convention "A port copies the bash, quirks included" in `docs/CONVENTIONS.md`, for the first
  release line only

## Blocked by

- None (can start immediately)

## Mode

with the human: DEV-one-agent-one-ticket allows no unattended work, and the change touches a rule
the parity harness holds.
