# Ticket: Read git's file list and grep's binary test once in the shell

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`Tree::read_text_files` and `Tree::read_markers` each list the files git does not ignore, keep
those `-f` finds, read their bytes, and sort them as `grep -I` does: a NUL byte in the first
32 KiB leaves a file out, a later one is not decided. The second copy came with the port of
`check-traceability.sh`, and the two have already drifted. `read_markers` reads git's listing as
bytes and refuses a path of no UTF-8 only where its file holds a marker, the fix of a review;
`read_text_files` reads it through `git::run`, which refuses the whole run on any output of no
UTF-8, so that a path git does not quote, as under `core.quotePath=false`, ends the run where the
bash reads on. Give the shell one function that classifies bytes as `grep -I` does and one way to
read git's listing, each listing keeping how its bash script asks git (`-z` or not), and decide
the case of `read_text_files` as the bash does.

## Acceptance criteria

- [ ] Given the suite as it stands, when the readers share the function, then every test passes
  unchanged and the parity harness shows no difference.
- [ ] Given a fixture repository with `core.quotePath=false` and a text file whose path is no
  UTF-8, when `nuc check` runs the rules of the area `docs`, then it reports what
  `check-docs.sh` reports, or is not decided with a message naming the path, as the human
  settles, and a test pins it.
- [ ] Given `src/shell/`, when the ticket lands, then the test of grep's first buffer is written
  once.

## Read first

- ADR-0009, ADR-0010 (git, its configuration kept), ADR-0011.
- The conventions on the shell reading for the rules and on the byte classes of a port.
- `collect_text_files` of `scripts/check-docs.sh` and the listing of
  `scripts/check-traceability.sh`.

## Blocked by

- docs/plan/revision-2026-10-31/01-shell-tree-module.md

## Mode

with the human — ADR-0007 keeps every ticket with the human, and the shell is the code the
threats aim at.
