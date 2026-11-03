# Ticket: List the tracked paths of the Project Layout through the shell's one listing

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Ticket 03 of the traceability revision gave the shell one listing of git's paths,
`Tree::listed`, read as bytes the way the bash script asks for them. `Tree::read_layout` still
asks `git ls-files --cached` through `git::run`. That call ends the run with "wrote no UTF-8"
when a tracked path is no UTF-8 under `core.quotePath=false`. Check 10 of `check-docs.sh` reads
on: `git ls-files --cached 2>/dev/null | cut -d/ -f1 | LC_ALL=C sort -u`. #21 left this open.

Let `read_layout` take its listing from the same reader, with the arguments check 10 passes.
`Tree::listed` fixes its arguments today to `--cached --others --exclude-standard`, so it needs a
mode for `--cached` alone. Read
the top-level parts from the bytes as `cut -d/ -f1` and `sort -u` in the C locale read them.
Where a finding would then name a path of no UTF-8, the run is not decided, as for the text
files. Also decide whether a git that fails here passes as an empty list, as the bash's
`2>/dev/null` does, or stays not decided under Q-1.

## Acceptance criteria

- [ ] Given a repository with a tracked path that is no UTF-8 and `core.quotePath=false`, and a
  Project Layout that lists the path's top-level part, when `nuc check` runs
  `docs/project-layout`, then it decides as check 10 does, and does not end with "wrote no UTF-8".
  The test returns early where the file system cannot hold the name, as on APFS.
- [ ] Given the same tree with the top-level part missing from the layout, when `nuc check` runs,
  then the run is not decided (exit 2), since the finding names a path `nuc` cannot spell.
- [ ] Given the suite and the parity harness, when the change lands, then both pass unchanged.

## Read first

- ADR-0010 and ADR-0025.
- The conventions *The shell reads, the rules decide* and *A port copies the bash, quirks
  included*.
- `src/shell/tree.rs`: `listed`, `read_text_files`, `read_layout`.
- The test `an_unquoted_path_that_is_no_utf8_is_read_and_refused_only_where_a_finding_names_it`
  in `tests/check.rs`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
