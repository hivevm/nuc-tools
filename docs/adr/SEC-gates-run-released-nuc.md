# SEC-gates-run-released-nuc: This repository's gates never run the `nuc` being built: the checks stay the template's bash, and the hooks run a released `nuc` that CI installs from checksums committed here

- **Status:** 🟢 accepted
- **Date:** 2026-11-18
- **Deciders:** Markus Brigl
- **Applies to:** `scripts/check-all.sh`, the hooks under `.githooks/`, the jobs of `.github/workflows/checks.yml`, the checksums and the version under `release/gate/`, `release/install-nuc.sh` as a gate runs it, the `minimum` of `nuc.toml`, and every place they name the tool that runs a check

## Context

This repository builds the tool that derived projects run as their gates, and it has gates of its
own. They stay off the `nuc` built from the change under review, as this repository decided from
its start: a broken or tampered build would pass itself, the silent pass Q-1 forbids, and a
compromised change would reach `main` through a gate it wrote (*Threats*). The decision before
this one let a released `nuc` replace the bash scripts only later, through an ADR that narrows
NUC-checks-are-bash §1 once `nuc` reaches parity for every check.

Template release v0.12.0 moves part of that later forward, for the hooks only.
[NUC-hooks-run-nuc](NUC-hooks-run-nuc.md) makes the five git hooks the shims of a `nuc-tools`
release, pinned by their blobs. Each hands its decision to `nuc hook`, and a hook without `nuc`
refuses. The checks the hooks run stay bash scripts under NUC-checks-are-bash §1. A take-up of
v0.12.0 here therefore has hooks that run `nuc`. The check of that earlier decision fails on
each of them, and on the self-test of the hooks, which asks `nuc` for its version.

A rehearsed take-up showed the second force. NUC-hooks-run-nuc §4 asks each job of CI that runs a
hook to install `nuc` from a release, verified against its checksums. The template's workflow does
it by this repository's Action. [SEC-actions-pinned-by-sha](SEC-actions-pinned-by-sha.md) pins
every action by commit SHA, kept current by Dependabot, which would move the pin of the Action to
the source commit of a release. That commit carries no checksums and installs nothing
([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)). The latitude of
NUC-hooks-run-nuc names the installer of a release, with its checksums committed, for a project
that pins by SHA. This repository keeps that installer in its tree, `release/install-nuc.sh`, the
script every release commit carries; it takes the tree's copy as the installer the latitude means.

A released `nuc` reaches a machine in three ways
([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)). The release commit carries
`SHA256SUMS` and `VERSION`, and `release/install-nuc.sh` installs the binary they name after
verifying its checksum. The Action `hivevm/nuc-tools@vN` runs that installer. The Feature
`ghcr.io/hivevm/features/nuc` comes from `hivevm/features`.

A hook resolves `nuc` through the `PATH`. In the Dev Container that is the Feature's release,
pinned by the lock. A contributor who ran `cargo install --path .` may have a build first on the
`PATH`, and the shims, which stay unchanged, cannot tell.

## Decision

We will run this repository's hooks on a released `nuc`, installed in CI from checksums committed
under `release/gate/`, and never run a gate on the `nuc` being built.

1. **Checks.** The checks are the template's bash scripts, as NUC-checks-are-bash §1 decides. A
   released `nuc` replaces them only through an ADR that narrows that clause under
   NUC-checks-are-bash §8. That ADR waits until a released `nuc` reaches parity for every check
   these gates run.
2. **Hooks.** The hooks are the pinned shims of NUC-hooks-run-nuc, which run the `nuc` on the
   `PATH`. They and the one version query of `scripts/test-git-hooks.sh` are the only lines of a
   gate that run `nuc`.
3. **CI.** Each job that runs a hook installs `nuc` before it, by
   `release/install-nuc.sh release/gate/SHA256SUMS release/gate/VERSION <directory>`, and puts that
   directory on the `PATH`. `release/gate/` holds the two files of one release, copied from its
   release commit and moved by a pull request. Its version is no older than the `minimum` of
   `nuc.toml`. No job installs `nuc` by this repository's Action, for the reason in the Context.
4. **Dev Container.** The Dev Container installs `nuc` by the Feature `ghcr.io/hivevm/features/nuc`,
   pinned by the lock ([NUC-container-hides-host-daemon](NUC-container-hides-host-daemon.md)).
5. **Never a build.** No gate runs `cargo run`, a binary under `target/`, `cargo install --path`, or
   any other build of the working tree. That holds in CI and in a script a gate runs, the installer
   of clause 3 included. The tests of the build run the build, `scripts/check-build.sh` and the
   parity harness `scripts/check-parity.sh`; they are not the gate's judge of it.
6. **Local gate.** A hook trusts the `nuc` on the contributor's `PATH`, as it trusts every
   `--no-verify`. CI is the gate a change cannot pass with a build of itself.

**Out of scope:**

- when the checks switch to `nuc`, and which release that pins;
- the gates of derived projects;
- how a contributor outside the Dev Container installs a release.

## Alternatives considered

- **No `nuc` in any gate until it reaches parity for every check, and stay on template v0.11.1**
  — what this repository decided first, accepted on 2026-10-31, with clauses 1, 3, 4, and 5 of
  this one for a later switch of the checks. The gates stay bash, and this repository never takes
  up the hooks it ships to every project. Its own gate would no longer be the one derived projects
  run.
- **The gates run the binary being built** — the change judges itself. A broken build passes, and
  a malicious one waves itself through.
- **The switch of the checks fixed at v0.1 now** — simpler, but it narrows NUC-checks-are-bash §1
  before a released `nuc` can stand in for every check. A v0.1 that does not cover every check
  would leave part of the gates unchecked.
- **The switch as a supersession of NUC-checks-are-bash §1** — the record's first wording of the
  later switch. No project ADR supersedes a `NUC-` clause; it narrows one
  ([NUC-base-is-fixed §1](NUC-base-is-fixed.md), NUC-checks-are-bash §8). The decision stayed; the
  form of the switch follows the fixed base.
- **Install `nuc` in CI by the Action, pinned by the SHA of its release commit and exempted from
  Dependabot.** One step and no committed files, but an exception to SEC-actions-pinned-by-sha
  that moves only by hand, and a pin a reviewer cannot tell from a moved one.
- **Fetch the release commit `pinned/vX.Y.Z` in CI and run its installer with its checksums.** The
  installer and the checksums would come from the release rather than from the change, at the cost
  of a fetch of a tag in every job. The change under review can alter the local installer as well
  as the committed checksums, but both stand in the diff a reviewer reads, as every gate script
  does.
- **Allow `nuc` as a command in every gate.** Simpler for the check, but any script of this
  repository could then hand a gate's decision to a `nuc` the job may not have installed from a
  release.
- **Have a check compare the version of the `nuc` on the local `PATH` with `release/gate/VERSION`.**
  A build carries the version of its `Cargo.toml`, so a build of the same version passes, and the
  check would run only after the hook.
- **Rewrite the shims to refuse a `nuc` that is no release.** NUC-hooks-run-nuc §1 and §2 keep the
  shims unchanged, and a shim cannot tell a release from a build either.

## Sources / Prior art

- [NUC-checks-are-bash](NUC-checks-are-bash.md): the checks, the hooks, the local gate, and the
  binary in place of the scripts.
- The decision before this one, accepted on 2026-10-31, which kept every gate off `nuc` until it
  reached parity and which this one restates for the hooks.
- [NUC-hooks-run-nuc](NUC-hooks-run-nuc.md): the hooks as pinned shims, clause 4, and its latitude
  for a project that pins actions by SHA.
- A rehearsed take-up of template v0.12.0: `scripts/check-gates.sh` failed on each hook and on the
  version query of the self-test of the hooks, and the action references check on the Action.
- Compiler bootstrapping, in which a compiler is built by its previous release rather than by
  itself.

## Consequences

- Positive: this repository runs the hooks it ships, so a fault in a shim or in `nuc hook` meets
  this repository's gate too, once `release/gate/` names the release (Q-1).
- Positive: CI judges a change by a `nuc` that was released and reviewed before it, installed by
  the installer and checksums the change carries (Q-1, Q-3).
- Negative: a release of `nuc` protects this repository only after a pull request moves
  `release/gate/` and the Feature's lock, two places for one release, and the hooks with
  `.githooks/nuc.blobs` when the shims change.
- Negative: the local gate runs whatever `nuc` comes first on the `PATH`; a contributor who
  installs a build of the working tree has hooks that judge with it until CI runs.
- Negative: the change under review carries the installer and the checksums CI installs from; a
  reviewer reads a change to `release/install-nuc.sh` or to `release/gate/` as a change to a gate.
- Negative: the bash scripts and their dependency on bash 4.4 stay here until the checks switch.
  The bash checks read every text file, the tests included, so a test assembles the references it
  writes, as `src/testing.rs` and `tests/support/` do.
- Follow-ups: the release flow moving `release/gate/` in the pull request that bumps the lock; the
  switch of the checks once a released `nuc` reaches parity.

## Enforcement

- `scripts/check-gates.sh` runs in `scripts/check-all.sh` and in the `gates` job of `checks.yml`. It
  reads `scripts/check-all.sh`, the hooks under `.githooks/`, `.github/workflows/checks.yml`, every
  script under `scripts/`, and `release/install-nuc.sh`. It does not read `scripts/check-build.sh`
  and `scripts/check-parity.sh`, which run the build's own tests. Nor does it read itself and its
  self-test. It fails on `cargo run`, on `cargo install --path`, on a path under `target/`, and on
  `nuc` run as a command (clauses 2 and 5). It skips the hooks `.githooks/nuc.blobs` pins, through
  `scripts/lib/pin.sh`, and the one line of `scripts/test-git-hooks.sh` that runs `nuc --version`
  (clause 2). It fails on a job of `checks.yml` that runs a self-test that runs a hook without a
  step before it that installs `nuc` from `release/gate/` (clause 3). Those self-tests are
  `scripts/test-git-hooks.sh`, `scripts/test-ready-for-main.sh`, and
  `scripts/test-normalize-remap.sh`. It fails where `release/gate/VERSION` is older than the
  `minimum` of `nuc.toml` (clause 3). Its self-test `scripts/test-check-gates.sh` cites this ADR.

**Not mechanically decidable:** the review holds these judgements:

- whether `release/gate/` holds the files of the release its version names (clause 3);
- whether a job that runs a hook in another way than the three self-tests installs `nuc` first, and
  whether a job puts the installed directory on the `PATH` (clause 3);
- a command assembled from variables, which the check, reading line by line, does not see
  (clause 5).
