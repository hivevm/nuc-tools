# ADR-draft-version-coupling: A project's `nuc.toml` names the minimum `nuc` and the rules turned on, the template release ships both, and `nuc` reads the release from the README

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** the format of `nuc.toml`, how `nuc` loads it and the README's `Template release` line, and every rule's default state

## Context

The template keeps the rules; the tool decides only what is mechanical and states no rule of its
own (*Non-Goals*). The template and its derived projects are coupled to `nuc` by a minimum
version in their `nuc.toml` (*Constraints*). A tool update never turns a green project red: a new
check ships off, and only a template release turns it on (Q-6). A missing or too old binary fails
closed with a message naming how to install it (G-5), and nothing passes when it could not decide
(Q-1). The assumptions leave open whether `nuc` learns the template release a project carries
from the README or from `nuc.toml`. [ADR-0005](0005-template-releases.md) already has every
repository name its release in the `Template release` line of its README, which check 14 of the
template holds; `nuc upgrade` needs that release as its starting point (G-4).

## Decision

We will have each project's `nuc.toml` name the minimum version of `nuc` and the rule IDs turned
on, both shipped by the template release, and have `nuc` read the release the project carries
from the `Template release` line of its README.

- **`nuc.toml`** lies at the root of the repository and holds the minimum version of `nuc` and the
  list of rule IDs turned on.
- **Default:** a rule is off until the `nuc.toml` of a template release lists its ID; an update of
  `nuc` alone turns nothing on.
- **Fail closed:** `nuc` stops with a non-zero exit and a message naming how to install or update
  it when `nuc.toml` is missing or unreadable, when its minimum version is above the running one,
  or when it lists a rule ID this `nuc` does not know. A missing binary is reported by the shim in
  `.githooks/`.
- **Template release:** read from the `Template release` line of the README; `nuc.toml` does not
  repeat it.

**Out of scope:** the field names and the rest of the format of `nuc.toml`; options a rule may
take; the shims themselves, which the template ships.

## Alternatives considered

- **The template release in `nuc.toml` as well** — easier to parse, but two statements of one fact
  in the same repository, free to drift apart.
- **A table in `nuc` from template release to the rules turned on** — the tool would decide which
  rules hold, which the non-goals exclude.
- **New rules on by default** — a tool update could turn a green project red, against Q-6.

## Sources / Prior art

- [ADR-0005](0005-template-releases.md): the `Template release` line.
- [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html): comparing the running version with the
  minimum.

## Consequences

- Positive: which checks hold stays the template's decision, carried by its release (Q-6); every
  case in which `nuc` cannot know what to check stops it (G-5, Q-1); the release is stated once.
- Negative / trade-offs: `nuc` parses one line of a Markdown file, whose form ADR-0005 and the
  template's check hold; a project can edit its own `nuc.toml` and turn a rule off, as it can edit
  its bash today.
- Follow-ups: the format of `nuc.toml`; how `nuc upgrade` moves the README line and `nuc.toml`
  together.

## Enforcement

Tests cited with `Verifies: ADR-draft-version-coupling` run `nuc check` on fixtures with
`nuc.toml` missing, unreadable, naming a higher minimum, and listing an unknown rule ID, and check
each exits non-zero with the message; one with a rule known to `nuc` but not listed checks that
the rule reports nothing.
