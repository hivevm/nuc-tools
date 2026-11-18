# Ticket: Propose how a release of nuc picks the part of its version

## What to build

NUC-releases-are-semver-tags §3 says how the template picks major, minor, or patch. No ADR and no
convention says it for the releases of `nuc` itself. The only related rule is
OPS-feature-in-hivevm-features, under which the interface `hivevm/features` takes a release by
changes only with a major.

The evidence is a rework. #102 prepared the shims' keys as v0.3.0 (db8c303). After it merged, the
maintainer chose a patch: the binary decides and answers as before, and only the shims' own
refusals gained a key. That release went out as v0.2.1 in a pull request of its own (#103, 509f24f)
and needed one more lock of the Feature (#104). v0.2.2 (#106) followed the same reasoning without
anyone writing it down.

Propose an ADR through the `propose-adr` skill. It decides which part each kind of change takes:

- a change to what a rule decides or to its key;
- a change to a hook's answer;
- a change to the shims;
- a new command or option;
- a change to the command line's output;
- a change to wording only.

It also states what holds before v1.0.0.

## Acceptance criteria

- [ ] Given the releases v0.2.0 to v0.2.2, when the proposed ADR is read against them, then each
  falls under exactly one of its clauses, and v0.2.1 and v0.2.2 are patches under it.
- [ ] Given the proposal, when `scripts/check-docs.sh` runs, then it passes, and the ADR stands as
  `proposed` until the human accepts it.

## Read first

- Touches: `docs/adr/`, `CONTRIBUTING.md` (its release section)
- The `propose-adr` skill; NUC-releases-are-semver-tags; OPS-feature-in-hivevm-features;
  API-command-line-with-measure; API-hook-contract-keyed-shims

## Blocked by

- None (can start immediately)

## Mode

with the human: an ADR is the human's to accept.
