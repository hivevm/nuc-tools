# ADR-0023: A project's `nuc.toml` names the minimum `nuc` and the rules turned on, the template release ships both, each rule checks the tree as the bash check of the release its README names, and the hooks run on defaults without it

- **Status:** 🟢 accepted
- **Date:** 2026-11-06
- **Deciders:** Markus Brigl
- **Applies to:** the format of `nuc.toml`, how `nuc` loads it and the README's `Template release` line, every rule's default state, every rule whose bash check differs between template releases, and the trees `tests/parity/corpus.toml` pins for each such release, and how `nuc hook` reads both
- **Supersedes:** [ADR-0014](0014-version-coupling.md)

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

A derived project gets the findings of the bash checks of the template release it carries (G-1).
Template releases change those checks: v0.6.0 changes check 8 of `scripts/check-docs.sh` so that
an old ADR flips to superseded only when its successor is accepted, and changes check 9 to name
a job with a matrix by its check runs; v0.7.0 reads the required checks and the Project Layout
from `CONTRIBUTING.md`. The projects do not move at once: the parity corpus pins the template at
v0.5.0, v0.6.0, v0.6.1, and v0.7.0, and derived projects on v0.6.1
([ADR-0015](0015-parity-harness.md)). A rule that follows only one release reports false
findings on the trees of another, or passes what that release fails. A rule ID stays stable
(G-6). The bash hooks read neither `nuc.toml` nor the release line, and the fixtures of the
template's hook self-test carry neither; the maintainer decided that a hook works without
`nuc.toml`, on defaults ([ADR-0021](0021-hook-contract.md)).

## Decision

We will have each project's `nuc.toml` name the minimum version of `nuc` and the rule IDs turned on,
both shipped by the template release, have `nuc` read the release the project carries from the
`Template release` line of its README, have each rule check the tree as the bash check of that
release does, and have `nuc` refuse a tree whose release such a rule cannot place; `nuc hook` alone
runs without `nuc.toml` on defaults and takes the newest release it knows where the line names none.

- **`nuc.toml`** lies at the root of the repository and holds the minimum version of `nuc` and the
  list of rule IDs turned on.
- **Default:** a rule is off until the `nuc.toml` of a template release lists its ID; an update of
  `nuc` alone turns nothing on.
- **Fail closed:** `nuc` stops with a non-zero exit and a message naming how to install or update
  it when `nuc.toml` is missing or unreadable, when its minimum version is above the running one,
  or when it lists a rule ID this `nuc` does not know. A missing binary is reported by the shim
  that hands the hook over.
- **Template release:** read from the first `Template release` line of `README.md`, as check 14
  reads it; `nuc.toml` does not repeat it. The value is a `vX.Y.Z` tag at its start, or
  `unreleased`.
- **The behaviour:** a rule knows the releases at which its bash check changed. It checks a tree
  as the latest of them at or before the release the tree carries. A rule whose bash check never
  changed does not read the release.
- **A later release:** a release newer than any this `nuc` knows is checked as the newest one it
  knows. The `nuc.toml` of a release that changes a check raises its minimum to a `nuc` that
  ports the change.
- **`unreleased`:** checked as the newest release this `nuc` knows.
- **Not decided:** when a rule turned on reads the release, `nuc` refuses a tree whose release
  is before v0.5.0, the earliest the rules port, and a tree without a `README.md`, whatever
  rules are turned on. It also refuses a tree whose README has no `Template release` line or a
  line that names no release, unless `docs/template-release` is turned on. That rule then fails
  the run with the finding the bash reports, and a rule that reads the release reports only the
  findings it reports on every release it knows: those no release could take back.
- **The corpus:** for each release at which a ported check changed, the parity corpus pins a
  tree that carries it.

- **Hooks:** `nuc hook` departs from the clauses above in these points only, stated here and
  nowhere else:
  - **Without `nuc.toml`:** every `hook/` rule is on and there is no minimum; *Fail closed* does
    not refuse the missing file, and *Default* does not keep the `hook/` rules off.
  - **With `nuc.toml`:** *Fail closed* holds in full: an unreadable file, a minimum above the
    running `nuc`, and an unknown rule ID each exit 2. Its list governs the `hook/` rules.
  - **The release:** *Not decided* does not hold. Where the README or its release line is missing
    or names no release, a hook decides as the newest release `nuc` knows; a release before
    v0.5.0 is decided as v0.5.0, the earliest the hooks port. A hook refuses nothing for the
    release; check 14 gates the line.
  - The missing binary is reported by the reference shims of
    [ADR-0021](0021-hook-contract.md).

**Out of scope:** the field names and the rest of the format of `nuc.toml`; options a rule may take;
the shims themselves, which ADR-0021 decides; template releases before v0.5.0, beyond
the hooks' clause; how `nuc upgrade` moves a project between releases (G-4).

## Alternatives considered

- **The template release in `nuc.toml` as well** — easier to parse, but two statements of one fact
  in the same repository, free to drift apart.
- **A table in `nuc` from template release to the rules turned on** — the tool would decide which
  rules hold, which the non-goals exclude.
- **New rules on by default** — a tool update could turn a green project red, against Q-6.
- **A new rule ID for each changed bash check** (`docs/adr-supersessions-2`). It fits the
  default of a rule off until a release turns it on, since the `nuc.toml` of v0.6.0 would turn
  the new ID on, and no rule would read the release. But the ID of the same rule would change
  with every release that touches its check, against G-6, and `nuc explain` would have to tell
  the two apart.
- **The hooks refuse without `nuc.toml` like every command** — one rule, but every project and
  every fixture without `nuc.toml` would refuse every commit, and hook parity could not hold.
- **Only the newest release.** The corpus floor would move to v0.6.0, and a project on v0.5.x
  would get the findings of a release it does not carry until it takes up v0.6.0. That
  contradicts G-1.

## Sources / Prior art

- [ADR-0005](0005-template-releases.md): the `Template release` line.
- [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html): comparing the running version with the
  minimum.
- The release notes of hivevm/nuc v0.6.0 and the diff of `scripts/check-docs.sh` between v0.5.0
  and v0.6.0.
- ESLint and Clippy, from memory and not checked here: both keep a rule's ID stable and gate
  changed behaviour behind configuration, ESLint through its config's `ecmaVersion`, Clippy
  through `msrv`. Here the carried release plays that part.

## Consequences

- Positive: which checks hold stays the template's decision, carried by its release (Q-6); every
  case in which `nuc` cannot know what to check stops it (G-5, Q-1); the release is stated once; one
  `nuc` checks every project of the corpus by the release it carries, and a rule keeps its ID across
  releases.
- Negative / trade-offs: the hooks and the checks read `nuc.toml` differently, and a hook runs on a
  tree `nuc check` refuses; `nuc` parses one line of a Markdown file, whose form ADR-0005 and the
  template's check hold; a project can edit its own `nuc.toml` and turn a rule off, as it can edit
  its bash today. A rule that changed carries both behaviours, together with tests and corpus trees
  for each. Without `docs/template-release` turned on, a tree with no readable release line is not
  decided by the rules that read it, so it exits 2 rather than 1.
- Follow-ups: the format of `nuc.toml`; how `nuc upgrade` moves the README line and `nuc.toml`
  together; when no derived project carries an old release any more, the behaviour of that
  release can go, with its corpus tree.

## Enforcement

Tests cited with `Verifies: ADR-0023` run `nuc check` on
fixtures with `nuc.toml` missing, unreadable, naming a higher minimum, and listing an unknown rule
ID, and check each exits non-zero with the message; one with a rule known to `nuc` but not listed
checks that the rule reports nothing. Others run `nuc hook` without `nuc.toml` and without a release
line and check that it decides as the newest release with every `hook/` rule on; with a
`nuc.toml` listing no `hook/` rule that it exits 0; with an unreadable `nuc.toml`, a higher
minimum, and an unknown rule ID that it exits 2; and with a release before v0.5.0 that it decides
as v0.5.0. They live in `tests/hooks.rs`, from the tracer of `docs/plan/hooks-port/` on.

The tests of the rules that read the release, `src/rules/docs_adr_supersessions.rs`,
`src/rules/docs_check_lists.rs`, `src/rules/docs_project_layout.rs`, and
`src/rules/docs_template_identity.rs`, and those of `tests/check.rs` and `tests/releases.rs` cite
this ADR with a `Verifies:` marker: a rule checks a tree on each release at which its bash check
changed as that bash check does, every rule of `tests/parity/releases.tsv` reads the release where
its bash check changed, and `nuc` refuses a tree whose release it cannot place. The
parity harness runs the template at each release at which a ported check changed
([ADR-0015](0015-parity-harness.md)). For check 8 a variant edits one file, while
the behaviour that differs between the releases needs two ADRs, so only the tests above tell the
two behaviours apart there.
