# OPS-nuc-version-by-change: A release of `nuc` takes the part of its version by what the change does to a project

- **Status:** 🟡 proposed
- **Date:** 2026-11-18
- **Deciders:** Markus Brigl
- **Applies to:** the version in `Cargo.toml` and `Cargo.lock` that a release of `nuc` moves, the release tag `vX.Y.Z` it names, and the pull request that prepares a release

## Context

[NUC-releases-are-semver-tags §3](NUC-releases-are-semver-tags.md) says how the template picks
major, minor, or patch. It decides the template's releases, not those of `nuc`, which this
repository ships as binaries, an Action, and through `hivevm/features` a Dev Container Feature
([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)). That ADR fixes one rule for
the version of `nuc`: the release as an interface, its tag, asset names, `SHA256SUMS`, and
attestations, changes only with a major. Nothing says which part a change to what `nuc` decides
or prints takes.

The gap cost a rework. The shims' keyed refusals were prepared as v0.3.0 (#102). After the merge
the maintainer chose a patch: the binary decides and answers as before, and only the shims' own
refusals gained a key. The release went out as v0.2.1 in a pull request of its own (#103).
v0.2.2 (#106), the key on each refusal of `nuc.toml`, was cut as a patch by the same reasoning,
which nobody wrote down.

What a project relies on is what decides the part. A project pins the Action and the Feature by the
major tag `v0`, and its `nuc.toml` names a `minimum`
([API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md)). Every release under
`v0` therefore reaches every project through the moved tag and Dependabot. A project's CI reads
the exit code and the findings on the standard output
([API-command-line-with-measure](API-command-line-with-measure.md)). Each finding ends in its key,
and a refusal of `nuc.toml` or of a shim ends in its key on the standard error
([API-config-refusals-keyed](API-config-refusals-keyed.md),
[API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md)). The wording around a key is
not what a project matches.

A port that agrees with the bash where it did not is a fix. The bash checks of the template are the
reference ([TEST-parity-harness](TEST-parity-harness.md)). Such a fix can make a rule report
where it was silent and turn a project's CI red, but the project's tree was already red under the
bash.

A local clone that fetched the template as a remote carries the template's tags `v0.3.0` to
`v0.12.0` in the same namespace `v*` as the releases of `nuc`. They are not on `origin`. Locally,
`git tag v0.3.0` fails, and a command that reads the last tag `v*`, as
`scripts/sensor-release-notes.sh` does, can read a template tag. This decision does not take that
up.

## Decision

We will take the part of each release's version of `nuc` by what its changes do to a project
that runs it.

1. **Patch.** A release is a patch when none of its changes alters what a project relies on:
   - a fix that makes a rule or a hook decide as the template's bash does. The fix holds at a
     template release `nuc` already ports, as the parity harness or a fixture table compares;
   - a change of a message's wording, `nuc explain`'s text included;
   - a key, a citation, or a field added to an existing message or refusal, its exit code and
     decision the same;
   - a change to the shims that keeps each hook's answers and adds or rewords only their own
     refusals;
   - a change to the documentation, the build, the tests, or the release tooling that leaves the
     interface of [OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md) unchanged.
2. **Minor.** A release is a minor when a change adds what a project may rely on, or alters what
   a rule or a hook decides. What a project relies on already stays:
   - a rule, a command, an option, an output format, a key of a new finding, or a hook port;
   - a template release whose checks `nuc` now ports;
   - a change to what a rule or a hook decides that is no fix of clause 1.
3. **Major.** A release is a major when one of its changes breaks what a project relies on:
   - a change to the interface that
     [OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md) holds, as that ADR
     decides;
   - a rule ID, a key, an option, or an exit code removed or given another meaning.

   Before `v1.0.0`, as SemVer item 4 allows, the second item is a minor. The first stays a major,
   as OPS-feature-in-hivevm-features decides, and so makes its release `v1.0.0`.
4. **The highest part wins.** A release that gathers changes of several parts takes the highest.
5. **When.** The author of the pull request that prepares a release must name in it the part
   and the clause of this decision that sets it. The reviewer checks that against the changes
   since the last tag.

**Out of scope:**

- when `nuc` reaches `v1.0.0`;
- the template's versions, which NUC-releases-are-semver-tags §3 decides;
- the version of the Feature, which `hivevm/features` decides;
- the template's tags in a local clone.

## Latitude

None.

## Alternatives considered

- **Every visible change to the output is a minor.** It is safer for a project that matches a
  message's wording. But each finding and refusal ends in its key so that no project needs the
  wording, and v0.2.1 and v0.2.2 would both have been minors for an appended key.
- **Every change to a decision is a minor, a parity fix included.** It flags each release that
  can turn a project red. But a parity fix agrees with the bash the project's checks already
  run, and the project was red there.
- **A break is a major even before `v1.0.0`.** It keeps `v0` free of breaks. But every rule ID or
  key retired while the port is still growing would cost a major, and each major moves the tag the
  projects pin, with a Dependabot pull request in every project.
- **No ADR, a line in `CONTRIBUTING.md`.** It costs less, but it constrains every future release,
  and a reader of a patch that added keys would take it for a mistake without the reason.

## Sources / Prior art

- [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html), items 4, 6, 7, and 8: before
  `1.0.0` anything may change; a patch is a backward-compatible fix; a minor adds
  backward-compatible function; a major breaks the public API.
- [Cargo's SemVer compatibility guide](https://doc.rust-lang.org/cargo/reference/semver.html),
  under which a change of `y` in `0.y.z` is incompatible.
- NUC-releases-are-semver-tags §3, the template's own rule for its parts.
- The releases v0.2.0, a minor by clause 4: the new command `nuc measure`, the ports of template
  releases v0.9.0 to v0.11.0, and parity fixes; v0.2.1 and v0.2.2, keys added to refusals:
  patches; and the rework of #102 and #103.

## Consequences

- Positive: the pull request that prepares a release names its part by a clause, so the choice of
  #102 is made before the merge and checked in review.
- Positive: v0.2.0 to v0.2.2 each take one part: v0.2.0 a minor by clauses 2 and 4, v0.2.1 and
  v0.2.2 a patch by clause 1.
- Positive: a break before `v1.0.0` takes a minor, so retiring a rule ID or a key while the port
  still grows costs no major and moves no pinned tag beyond `v0`.
- Negative / trade-offs: before `v1.0.0` a minor may break, and the moved tag `v0` brings it to
  every project that runs the Action at once; the Feature's lock holds a Dev Container back until
  its Dependabot pull request merges. The notes of the release, which every hand-off feeds through
  "A derived project acts on", carry what a project does.
- Negative / trade-offs: a parity fix released as a patch can turn a project's CI red where its tree
  was red under the bash.
- Follow-ups: where the procedure of a release of `nuc` is written down, with the step that names
  its part; the Cutting a release section of `CONTRIBUTING.md` describes the template's.
- Follow-ups: the template's tags in a local clone that fetched the template as a remote, which
  share `v*` with the releases of `nuc`; how a clone fetches the template without its tags.

## Enforcement

**Not mechanically decidable:** the part a change takes depends on what it does to a project. No
check reads that from a diff. `release/version.sh` holds the tag to the version of
`Cargo.toml`, not the part. The review of a release's pull request checks the part it names
against clauses 1 to 4 and the changes since the last tag.
