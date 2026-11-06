# Ticket: Decide the hook contract and hook-scenario parity

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The two decisions every later ticket builds on, as proposed ADRs worked out with the human
(`propose-adr`), before any hook code:

- **Hook-scenario parity:** a harness of scenarios through git, each run with the bash hooks of a
  pinned template release and with the reference shims and `nuc`, comparing exit status, mapped
  refusals, and the notices a hook writes, failing on any difference; which release's hook
  self-test supplies the scenarios; how the nuc side gets what it reads and the bash side does not.
- **Hook contract:** what `nuc hook <name>` decides and what the shim runs; how `nuc` tells the
  shim to run or skip the checks and how the shim has a passed run recorded, the tree fixed before
  the checks ran; that a shim refuses on any exit code the contract does not name; the reference
  shims in POSIX sh and where they live; and how the `hook/` rules stand to ADR-0014, its opt-in
  list, its refusals of a missing `nuc.toml` and of a tree without a readable release line, and
  what a hook reads from that line. Where it widens ADR-0013's exit codes or departs from ADR-0014,
  it supersedes that clause.

## Acceptance criteria

- [ ] Given this ticket closes, when its branch merges, then both drafts are in `docs/adr/` with
  their index rows, proposed, each with an Enforcement that names the test of a later ticket.
- [ ] Given a fixture of the template's hook self-test, which has no `nuc.toml` and no release
  line, when the contract is read, then it says what `nuc hook pre-commit` does there, so that the
  scenario of a commit on a branch can agree with the bash.
- [ ] Given the drafts, when the human has reviewed them, then the commit they reviewed is named,
  for the `ADRs reviewed at:` line of the tracer's pull request.

## Read first

- Touches: `docs/adr/`.
- The spec next to this ticket, sections *Implementation decisions* and *ADR impact*.
- ADR-0013, ADR-0014, ADR-0015, ADR-0016, ADR-0007; Q-1, Q-2, Q-5, Q-7 in
  `docs/SPECIFICATION.md`.
- The template's `.githooks/` and `scripts/test-git-hooks.sh` at each pinned release, from the
  parity cache.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADRs are proposed and reviewed with the human
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules)), and ADR-0007 allows no unattended work.
