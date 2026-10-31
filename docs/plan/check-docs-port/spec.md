# Spec: Port check-docs.sh to `nuc check`

<!-- Copy to docs/plan/<spec>/spec.md. The destination of a piece of work larger than one agent
     session, written from the user's side; its tickets lie next to it in the same directory. It
     is disposable: the design revision deletes it once its tickets have merged, and whatever has
     to last moves into docs/SPECIFICATION.md, an ADR, docs/ARCHITECTURE.md, docs/GLOSSARY.md, or
     docs/CONVENTIONS.md first. No file paths or code; they go stale. How it is written is the
     plan-feature skill under .agents/skills/. Decided in docs/adr/0006-feature-layer.md. -->

## Problem

A derived project checks its documents with the template's `check-docs.sh`: about 1,300 lines of
bash, which needs bash 4.4 or later and the GNU tools, so it fails on the bash macOS ships and
does not run on Windows outside Git Bash. Its sixteen checks report messages without a rule ID,
so a finding cannot be explained or turned off on its own. `nuc check` ports one of them so far:
the Template release line.

## Solution

A maintainer of a derived project on template release v0.5.0 or later runs `nuc check` and gets
the same findings, by rule ID and `file:line`, as the `check-docs.sh` of v0.5.0, on any of the
platforms of Q-4. Each check is one rule that `nuc explain` gives the reason of. The parity
harness shows it on the template at v0.5.0 and on a derived repository of that release, on their
trees as pinned and on variants of them that make each ported check fire.

## Existing behaviour

- `nuc check` runs the rule `docs/template-release` (check 14 of `check-docs.sh` at v0.5.0) and
  refuses as the command-line contract decides; `tests/check.rs` and `tests/contract.rs` pin it.
- The parity harness compares bash and `nuc` on the template at v0.5.0 only; its self-test pins
  the comparison.
- This repository's own gates keep running the bash `check-docs.sh` (ADR-0018); derived projects
  keep running theirs. Nothing switches over in this spec.

## Goals served

- G-1, for every check of `check-docs.sh`; G-6, a rule ID and `file:line` per finding and its
  reason; Q-1, every input a check cannot read refuses; Q-2, parity on the corpus; Q-7, the
  checks read and never run repository content; Q-11, no network.

## User stories

1. As the maintainer of a derived project on v0.5.0, I want `nuc check` to report what
   `check-docs.sh` of that release reports, so that I can rely on either.
2. As that maintainer, I want each finding to name its rule ID and `file:line`, and
   `nuc explain` to give the rule's reason, so that I can fix it without reading the bash.
3. As that maintainer on macOS or Windows, I want the checks to run without bash 4.4, so that the
   local gate works where my machine does.
4. As the maintainer of nuc-tools, I want the parity harness to compare bash and `nuc` on
   failing trees too, so that agreement means more than both passing a clean tree.
5. As the maintainer of nuc-tools, I want the harness to refuse a corpus tree of a release before
   v0.5.0, so that a pin moved to an older release cannot slip into the comparison.
6. As the reviewer of a ticket, I want each ported check to arrive with the bash self-test's cases
   as its unit tests, so that the port is decided case by case.

## Implementation decisions

- **One rule per check, as of v0.5.0.** Each check of `check-docs.sh` at v0.5.0 becomes one rule
  module with an `area/name` ID in the area `docs`. A later release that changes a check brings a
  rule ID of its own, since a rule ID never changes meaning, and its `nuc.toml` turns it on
  (ADR-0013).
- **Inputs read by the shell.** The shell reads what a check needs, before any rule runs, as
  values: the tracked and untracked, not ignored, files of the tree through git; the texts of the
  Markdown files, the ADRs, the README, the workflows, and `scripts/check-all.sh`; the skill
  directories and their pointers with whether each is a symlink and where it resolves; the
  attributes that mark a file generated; and, for the check of drafts on `main`, the branch a
  change lands on from the environment variables the bash reads. A git command that starts being
  used brings its neutralized keys (ADR-0010).
- **Line-based reading, as the bash reads.** Markdown is read line by line with the bash's rules
  for fences, front matter, tables, and headings, not as CommonMark; the port copies the bash's
  exemptions and oddities ([conventions](../../CONVENTIONS.md)).
- **Messages.** The wording may differ from the bash's; the rule ID, file, and line are what the
  normalization table maps and compares (ADR-0019).
- **Corpus.** Every repository on template release v0.5.0 or later, each at a fixed commit: the
  template at v0.5.0, `mbrigl/opamp-fleet`, `hivevm/nectar`, and this repository,
  `hivevm/nuc-tools`. Each entry names its template release, and the harness refuses a tree whose
  README names a release before v0.5.0. A later release whose bash
  differs shows as a difference, and its behaviour becomes a rule ID of its own.
- **Variants.** For each ported rule, the harness derives one or more variants of a corpus tree
  by a scripted edit, such as a broken link or a missing index row, and compares bash and `nuc`
  on each as on the tree itself.

## Testing decisions

- Each rule's unit tests copy the cases of the bash self-test, `test-check-docs.sh`, for its
  check, as the golden path's rule does.
- The integration seam is the golden path's: a fixture repository built through git, `nuc` run
  on it as a process, where a check reads git or the file system in a way a unit test cannot show.
- Parity on the corpus and its variants decides each ticket: a ticket is done when the harness
  shows no difference for the rules it ports. The harness's self-test gains cases for the
  release an entry names and for variants.

## Out of scope

- The other scripts of the template: `check-traceability.sh`, `check-devcontainer.sh`,
  `check-shell.sh`, the sensors, the hooks, the merge driver, and the scripts run by hand; each
  is a spec of its own, in the order the setup named.
- Switching this repository's gates or a derived project's to `nuc` (ADR-0018).
- `--format json` and `--format sarif` (G-7); `nuc upgrade` (G-4).
- Projects on template releases before v0.5.0, and their behaviour of a check.
- Shipping `nuc.toml` with a template release; that is the template's change.

## ADR impact

- Binding: ADR-0009 (the shape), ADR-0010 (git), ADR-0011 (fixtures),
  ADR-0020 (parity), ADR-0013 (version coupling), ADR-0018 (the
  gates), ADR-0019 (the contract).
- **Variants and the release floor:** ADR-0012 compared on the pinned trees only. The proposed
  ADR-0020 supersedes it, adding the variants and the corpus of
  releases v0.5.0 and later; it is accepted before ticket 01 lands.
