# Ticket: Carry the wording of two accepted ADRs into the next normalization

## What to build

Accepted ADRs are restated only by a normalization, so this ticket keeps the list until one runs.
The reviews of `nuc measure` found where two accepted ADRs no longer say what the system does:

- **API-measure-reads-harness-transcripts:**
  - Its Context says a request's lines repeat its usage, and its Decision says a request counts
    once "however many lines repeat its usage". Over all projects of the maintainer's machine, 387
    of 129,741 requests report a usage that grows while they stream, so the count takes each
    field's largest.
  - It says one function of the shell reads outside the repository. Two do, named in
    `tests/functional_core.guarded`: one lists, one reads.
  - It names the transcripts as what is read. The shell reads each subagent's `agent-*.meta.json`
    beside them too.
  - It says the current branch always counts. To measure other branches alone, run it on a
    branch without sessions or on a detached HEAD, as the README says. The maintainer chose to
    document this and keep the decision.
- **API-output-formats-carry-reason:** it says `nuc` writes JSON "without a crate". That still holds
  for the writer, but `serde_json` is now a dependency for reading transcripts, so the sentence
  reads
  as if `nuc` had none.

## Acceptance criteria

- [ ] Given a normalization the human has instructed, when it restates the two ADRs, then each says
  what the list above says the system does, and changes no decision.

## Read first

- Touches: `docs/adr/` (in the normalization only)
- The `normalize` skill; the mechanics in `docs/adr/README.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: a normalization runs on the human's instruction alone.
