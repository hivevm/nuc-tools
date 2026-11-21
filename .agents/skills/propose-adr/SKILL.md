---
name: propose-adr
description: Develop and submit an Architecture Decision Record, with a catalogue of concepts to propose from. Use when a change is architecture-relevant, costly to reverse, constrains future choices, or would look like a mistake to a reader without this conversation, when it takes an early decision no ADR decides, such as the shape of the code, the environment, or the supply chain, or when the human asks for an ADR.
---

# Propose an ADR

The rules are
[NUC-decisions-are-proposed-then-accepted](../../../docs/adr/NUC-decisions-are-proposed-then-accepted.md),
[NUC-adrs-are-named-by-category](../../../docs/adr/NUC-adrs-are-named-by-category.md), and, for
the `Extends`, `Narrows`, and `Role` fields,
[NUC-base-is-fixed](../../../docs/adr/NUC-base-is-fixed.md); the mechanics of the record are in
[`docs/adr/README.md`](../../../docs/adr/README.md). This skill is the order of work. It states no
rule of its own ([NUC-procedures-are-skills §3](../../../docs/adr/NUC-procedures-are-skills.md)).

## 1. Calibrate

Decide with
[NUC-decisions-are-proposed-then-accepted §2](../../../docs/adr/NUC-decisions-are-proposed-then-accepted.md)
whether this needs an ADR at all. The rule's third case, what a reader without this conversation
would take for a mistake, is the deliberate deviation from the obvious path; record it, or the next
session "fixes" it. When no ADR is needed, say so and stop here.

## 2. Interrogate the request

Before writing a line of the record, ask the human, in rounds until the answers stop changing the
decision, at least:

- Which goal or quality goal of [`docs/SPECIFICATION.md`](../../../docs/SPECIFICATION.md) does
  this serve, by identifier (`G-n`, `Q-n`)?
- What did the human not say that the decision depends on?
- Which alternative has nobody mentioned yet?
- What happens if no decision is made?
- Does this bind future choices, or is it a *how* that needs no ADR?
- Would someone who lacks this conversation see the result and try to fix it?

Question the framing and your own draft. A draft written ahead of the answers is the first
alternative to reject.

When the catalogue has an entry for the decision, read it in
[`concepts/README.md`](concepts/README.md) before the first round. Put its candidates to the
human:

- the two or three whose *Fits when* matches the quality goals;
- what each one's *Does not fit* says about this project;
- the simplest form, as one of them.

Between candidates that fit equally, prefer the one whose rule a test or check can decide: what
a computational sensor holds needs no reviewer to remember it. The human chooses; a candidate
nobody chose is an alternative in the record.

## 3. Write the record

1. Copy [`docs/adr/template.md`](../../../docs/adr/template.md) to `<CAT>-<slug>.md` and head it
   `# <CAT>-<slug>: <title>`. The identifier is picked now and stays
   ([mechanics](../../../docs/adr/README.md#mechanics)). Complete the header and delete the
   instruction paragraph.
2. One decision per ADR: one "We will …" sentence. A second one is a second ADR, unless the
   decision is the interplay of an integration
   ([NUC-decisions-are-proposed-then-accepted §1](../../../docs/adr/NUC-decisions-are-proposed-then-accepted.md)).
3. **Alternatives:** every one the interrogation surfaced, each with why not. For a concept of
   the catalogue, its entry's sections carry over: *Fits when* and *Does not fit* into the
   Context, *Alternatives* and *Sources* as they stand, checked against this project, and *The
   rule a test decides* into the Enforcement.
4. **Sources:** what was consulted. A decision costly to reverse is researched first, external
   sources included, and nothing recalled is passed off as verified
   ([§4](../../../AGENTS.md#4-working-style)).
5. **Consequences:** weighed against the specification's quality goals. Follow-ups by topic,
   never by an identifier that does not exist yet.
6. **Enforcement:** the test that will cite this ADR with a `Verifies:` marker, or a paragraph
   starting `**Not mechanically decidable:**` with the reason
   ([NUC-decisions-cite-tests §4](../../../docs/adr/NUC-decisions-cite-tests.md)).
7. Add the index row in [`docs/adr/README.md`](../../../docs/adr/README.md), status and *Applies
   to* mirrored verbatim. For a supersession, name the old ADR in `Supersedes` and leave its
   status line: the human flips it when accepting this one
   ([mechanics](../../../docs/adr/README.md#mechanics)).
8. Run [`scripts/check-docs.sh`](../../../scripts/check-docs.sh).

## 4. Submit

Run [`scripts/sensor-plain-rules.sh`](../../../scripts/sensor-plain-rules.sh) by itself and split
each long sentence it names in the ADR. `--require` passes a long sentence, and after the
acceptance only a superseding ADR shortens it. Do the same after each revision of the ADR.

Commit on a branch and hand the ADR to the human as
[NUC-decisions-are-proposed-then-accepted §3](../../../docs/adr/NUC-decisions-are-proposed-then-accepted.md)
asks. What follows is
[NUC-decisions-are-proposed-then-accepted §4](../../../docs/adr/NUC-decisions-are-proposed-then-accepted.md).

## 5. The acceptance of a superseding ADR

When the human accepts an ADR whose `Supersedes:` field names others, each old ADR flips to
superseded in the same change
([NUC-adrs-are-named-by-category §4](../../../docs/adr/NUC-adrs-are-named-by-category.md)). From
then on, each `Verifies:` marker that cites an old ADR fails
([NUC-decisions-cite-tests §6](../../../docs/adr/NUC-decisions-cite-tests.md)). So the same change
moves the citations to the successor:

1. **The map.** Write a map under `.scratch/` from the `Supersedes:` fields, in the form the
   header of the map script [`remap.sh`](../normalize/remap.sh) gives. Each old ADR gets a line
   `<old> <successor>`. A cited clause that the successor carries under another number gets a
   line `<old>§n <successor>§m` of its own.
2. **The move.** Check the map with `.agents/skills/normalize/remap.sh --check <map>`, then apply
   it without `--check`. Rewrite by hand what it lists as left. The script drops an old identifier
   from a marker that already names the successor. It reads no file under `docs/adr/`, and an
   accepted ADR keeps its citations
   ([NUC-decisions-are-proposed-then-accepted §6](../../../docs/adr/NUC-decisions-are-proposed-then-accepted.md)).
3. **The width.** A longer successor may push a line past the width. Split such a line only
   within its comment or its paragraph, never into the line that follows it. Keep each marker on
   one line: the traceability check reads no marker split over two lines. Leave every other line
   as it was.
4. **The comparison.** Compare each changed file with the base before the commit. This loop
   prints each file whose marker lines name fewer identifiers than at the base:

   ```bash
   ids() { grep Verifies: | grep -oE '[A-Z]{2,5}-[a-z0-9]+(-[a-z0-9]+)+|[GQ]-[0-9]+' | wc -l; }
   git diff HEAD --name-only | while read -r f; do
     [ "$(git show HEAD:"$f" | ids)" -le "$(ids < "$f")" ] || echo "$f"
   done
   ```

   A file it prints lost an identifier from a marker. Either the script dropped it beside its
   successor, or a split moved it off the marker's line, where the traceability check no longer
   reads it. Check each such file by hand before the commit. Its count of words, `wc -w`, may fall
   only by the identifiers the script dropped.
