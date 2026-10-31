# Spec: Port check-traceability.sh to `nuc check`

<!-- Copy to docs/plan/<spec>/spec.md. The destination of a piece of work larger than one agent
     session, written from the user's side; its tickets lie next to it in the same directory. It
     is disposable: the design revision deletes it once its tickets have merged, and whatever has
     to last moves into docs/SPECIFICATION.md, an ADR, docs/ARCHITECTURE.md, docs/GLOSSARY.md, or
     docs/CONVENTIONS.md first. No file paths or code; they go stale. How it is written is the
     plan-feature skill under .agents/skills/. Decided in docs/adr/0006-feature-layer.md. -->

## Problem

A derived project holds its decisions and criteria to tests with the template's
`check-traceability.sh`: bash that needs bash 4.4 or later and the GNU tools, like
`check-docs.sh` before its port. Its findings carry no rule ID, so a finding cannot be explained
or turned off on its own. `nuc check` ports every check of `check-docs.sh` and none of this one.

## Solution

A maintainer of a derived project on template release v0.5.0 or later runs `nuc check` and gets
the same findings as `check-traceability.sh` of that release, by rule ID and `file:line`, on any
of the platforms of Q-4:

- the criteria of the specification, each with a bold identifier in its section and none twice;
- every marker citing an ADR or criterion that exists and is in force;
- every accepted ADR cited, or declared not mechanically decidable with a reason.

The criteria no test cites yet are named in the run's summary, as the bash names them, and never
as a finding. The parity harness shows the agreement on the corpus and on variants that make each
rule fire.

## Existing behaviour

- **The script is the same in every release.** `check-traceability.sh` is byte for byte the same
  at v0.5.0, v0.6.0, and v0.6.1. No rule of this spec depends on the carried release.
- **`nuc check` today.** It runs the sixteen rules of the area `docs`. `tests/check.rs` and the
  rules' unit tests pin them.
- **The harness today.** It runs only `check-docs.sh` on each corpus tree and variant, and
  compares that against `nuc check`. Its self-test pins the comparison.
- **What does not switch.** This repository's gates keep running the bash (ADR-0018), and derived
  projects keep running theirs.

## Goals served

- **G-1:** the findings of `check-traceability.sh`.
- **G-6:** a rule ID and `file:line` per finding, and the rule's reason.
- **Q-1:** every input the check cannot read refuses.
- **Q-2:** parity on the corpus, now for two bash scripts.
- **Q-7:** markers are read, never run.
- **Q-11:** no network.

## User stories

1. As the maintainer of a derived project, I want `nuc check` to report what
   `check-traceability.sh` of my release reports, so that one command holds both the documents
   and their traceability.
2. As that maintainer, I want a citation of a superseded ADR to name its file and line, so that I
   can move the marker to the ADR in force.
3. As that maintainer, I want the criteria still without a test listed apart from the findings,
   as the bash lists them, so that pending work never fails a run.
4. As that maintainer, I want `nuc explain` to give each traceability rule's reason, so that I
   can fix a finding without reading the bash.
5. As the maintainer of nuc-tools, I want the harness to run both bash checks of a tree and
   compare them with one `nuc check`, so that parity covers every ported rule.
6. As the reviewer of a ticket, I want each rule to arrive with the bash self-test's cases as its
   unit tests, so that the port is decided case by case.

## Implementation decisions

- **Three rules, in the area `traceability`.** The script's three parts become one rule each:
  - the criteria of the specification;
  - the citations of the markers;
  - the coverage of the accepted ADRs, including the exemption's missing reason.
- **Rules ship off.** A new rule ships off, and a template release's `nuc.toml` turns it on
  (Q-6, ADR-0013). Nothing turns them on in this spec.
- **Inputs read by the shell.**
  - The specification's text and the ADRs, as the docs rules already read them.
  - Every file git does not ignore, tracked or new, that is not Markdown and holds a marker.
  - A file the bash's grep takes for binary is skipped, as the bash skips it.
  - A file whose bytes the port cannot read as the bash does refuses (not decided), as the
    conventions on byte classes say.
- **The pending criteria.** The criteria no marker cites go into the summary `nuc check` writes
  to standard error, as the bash's closing line names them. They are no finding and never change
  the exit status.
- **Messages.** The wording may differ from the bash's. The rule ID, file, and line are what the
  normalization table maps and compares (ADR-0019).
- **The harness runs every ported bash check.**
  - It runs both `check-docs.sh` and `check-traceability.sh` of a tree.
  - It compares their findings together with those of one `nuc check`.
  - A tree fails when either script fails.

## Testing decisions

- Each rule's unit tests copy the cases of `test-check-traceability.sh` for its part, as the docs
  rules copy theirs.
- The integration seam is a fixture repository built through git, with `nuc` run on it as a
  process, where the rule reads git or the file system in a way a unit test cannot show: ignored
  files, new files, binary files, a missing specification.
- Parity on the corpus and its variants decides each ticket. Each rule gains variants that make it
  fire: a marker citing a missing or a superseded ADR, a list item without its identifier, an
  accepted ADR left uncited.

## Out of scope

- **The hooks, the merge driver, the shims, and the latency of `reference-transaction`.** They are
  the next spec (G-2, G-3, G-5, Q-5).
- **The other bash scripts.** `check-devcontainer.sh`, `check-shell.sh`, `check-all.sh`, the
  sensors, and the scripts run by hand.
- **Turning the rules on.** Shipping a `nuc.toml` that turns the rules on is a template release's
  change.
- **Switching gates to `nuc`.** Neither this repository's gates nor a derived project's switch
  (ADR-0018).
- **Output formats.** `--format json` and `--format sarif` (G-7) are not part of this spec.

## ADR impact

- **Binding ADRs:**
  - ADR-0009 (the shape);
  - ADR-0010 (git);
  - ADR-0011 (fixtures);
  - ADR-0013 (version coupling);
  - ADR-0018 (the gates);
  - ADR-0019 (the contract);
  - ADR-0022 (parity);
  - ADR-0023 (the decision the check holds).
- **No new ADR.** ADR-0022 already compares "the bash scripts" of a tree, which covers a second
  script, and the check does not change between releases, so ADR-0021 has nothing to decide here.
