# Ticket: Keep the bash tools' readings of bytes in one module

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

How the bash's tools read bytes is spread over the crate: `adr::awk_lines`, `adr::read_lines`
(re-exported as `rules::read_lines`), `docs_check_lists::sed_lines`, `markdown::read_lines`,
`adr::is_space` and a second `is_space` over `char` in `docs_template_release`, `adr::trim`,
`adr::first_word`, `adr::find`, and `adr::contains`. The rules of links, section references,
check lists, and the Project Layout take them from `src/adr.rs`, the reading of the record. The
byte divergences the reviews found, a last line `read` drops, the space class, were each fixed in
one of these copies. Gather them into one module named for the bash's tools, with one test each
of the bytes it reads otherwise than Rust, and leave `src/adr.rs` the reading of the record.

## Acceptance criteria

- [ ] Given the suite as it stands, when the readers have moved, then every test passes unchanged
  and the parity harness shows no difference.
- [ ] Given the crate, when a rule needs lines as awk, `read`, or `sed` split them, or POSIX
  space, then it takes them from the one module, and `src/adr.rs` defines none of them.
- [ ] Given `docs/ARCHITECTURE.md`, when the module lands, then the Rules block names it.

## Read first

- ADR-0009.
- The conventions on porting the bash with its quirks.
- `src/adr.rs`, `src/markdown.rs`, and the users of the functions above.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
