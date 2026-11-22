# OPS-coverage-names-unrun-lines: A sensor of the checks names the lines of `src/` a change adds that no test runs, over cargo-llvm-cov pinned in the Dev Container

- **Status:** 🟡 proposed
- **Date:** 2026-11-22
- **Deciders:** Markus Brigl
- **Applies to:** the sensor of the lines no test runs under `scripts/` and its self-test, its line in `scripts/check-all.sh` and its step in the `sensors` job of `checks.yml`, the installer of cargo-llvm-cov under `.devcontainer/` and its line in `devcontainer.json`, the `check-cfg` of `cfg(coverage)` in `Cargo.toml`, and the cleared environment of the test fixtures in `tests/support/`

## Context

The design revision of 2026-11-22 read the cost of the work on `nuc upgrade`. Each of its five
tickets took two rounds of review. In three of them, the findings of the first round were branches
no test ran: refusals no run of the binary decided, the plan deleting `README.md`, and the failure
branches of the replay. In a fourth, a unit test ran the de-duplication and did not decide it,
which a report of unrun lines does not show. A round of review costs a run of
`scripts/check-all.sh`, 923 s in the last run. The tickets of that revision showed the same: the
reviewers found untested branches by mutants, by hand.

A report of the lines a change adds that no test runs shows these branches before the review. The
cause is in the project's own checks, not in the template's.

[OPS-rust-stable-behind-scripts](OPS-rust-stable-behind-scripts.md) names the sensors by kind and
has none for coverage. Its toolchain holds the components `rustfmt` and `clippy` alone. The Dev
Container installs ShellCheck by a script that pins its version and checksums
(`.devcontainer/install-shellcheck.sh`), and cargo-deny and cargo-audit by `cargo install --locked`
at fixed versions
([OPS-dev-container-carries-and-reaches](OPS-dev-container-carries-and-reaches.md)).

cargo-llvm-cov wraps the instrumentation of rustc (`-C instrument-coverage`) and `llvm-cov`. It
needs the rustup component `llvm-tools-preview`. It writes `lcov` among other formats, and builds
with `cfg(coverage)` set. Its releases carry archives for Linux, macOS, and Windows, x86_64 and
aarch64, without checksum files. Since 0.8.5 they carry an artifact attestation; the latest is
v0.9.2.

The contract tests run the built `nuc` as a process under a cleared environment, which passes only
`PATH` and `SystemRoot` ([TEST-fixtures-with-own-home](TEST-fixtures-with-own-home.md)). An
instrumented binary writes its profile where `LLVM_PROFILE_FILE` names, by default into its
current directory. Cleared of the variable, the binary run by a test writes that file into the
fixture's root. The snapshots and the git status the tests compare then differ, and the profile is
lost with the fixture.

[NUC-checks-are-bash §7](NUC-checks-are-bash.md) keeps three kinds of script out of
`scripts/check-all.sh`; a sensor whose run costs less than a full run of the checks is none.
[NUC-checks-are-bash §5](NUC-checks-are-bash.md) has `scripts/check-all.sh` and `checks.yml` run the
same checks, which the rule `docs/check-lists` holds; every sensor of the checks is a step of the
`sensors` job. [OPS-dev-container-carries-and-reaches §2](OPS-dev-container-carries-and-reaches.md)
pins a tool the checks run the same in the container and in CI. The instrumented test run costs
about what `cargo test` does, in a target of its own. The build, test, and lint check takes 13 s of
the 923 s on an incremental build.

The bash under `scripts/lib/` has no such tool in place, and none was asked for.

The maintainer chose a sensor that reports and never fails on what it reports, for the Rust
sources, as a line of `scripts/check-all.sh`, with the tool pinned in the Dev Container. The two
clauses above make it a step of CI as well, with the tool pinned there by the same installer.

## Decision

We will have a sensor of the checks, over a pinned cargo-llvm-cov, name the lines of `src/` a
change adds that no test runs.

- **What it reports:** `scripts/sensor-unrun-lines.sh` names each line the change adds under
  `src/` that the `lcov` report of cargo-llvm-cov counts as run zero times. The change is the diff
  against the merge base with the remote's `main`, or against a base it is given. It prints one
  line per such line, `<file>:<line>`, and a count. It exits 0 whatever it reports. It exits 2,
  naming the cause, where cargo-llvm-cov or `llvm-tools-preview` is missing or the instrumented
  test run fails, and reports nothing then (Q-1).
- **Where it runs:** as a `run` line of `scripts/check-all.sh`, in the form of
  [NUC-checks-are-bash §4](NUC-checks-are-bash.md), its self-test
  `scripts/test-sensor-unrun-lines.sh` beside it. It is also a step of the `sensors` job of
  `checks.yml`, as [NUC-checks-are-bash §5](NUC-checks-are-bash.md) asks.
- **The tool:** `.devcontainer/install-cargo-llvm-cov.sh` installs cargo-llvm-cov at one version
  from its release page, and the component `llvm-tools-preview` through rustup. It refuses an
  archive whose SHA-256 is not the one it holds for its machine, as
  `.devcontainer/install-shellcheck.sh` does. The checksum decides, not the attestation.
  `devcontainer.json` runs it on creation, and the `sensors` job runs it before the sensor, as both
  run the installer of ShellCheck. The version stands in the installer alone.
  `rust-toolchain.toml` stays as it reads. `Cargo.toml` names `cfg(coverage)` in the `check-cfg` of
  its `unexpected_cfgs` lint, which `-D warnings` would otherwise fail on.
- **The fixtures:** built with `cfg(coverage)`, the cleared environment of `tests/support/` passes
  `LLVM_PROFILE_FILE` through where it is set. That build alone departs from the allowlist
  [TEST-fixtures-with-own-home](TEST-fixtures-with-own-home.md) gives the fixtures. Built otherwise,
  it passes `PATH` and `SystemRoot` alone, as before.

**Out of scope:**
- coverage of the bash under `scripts/`, and of what the parity harness runs outside `cargo test`;
- a threshold or a gate;
- mutation testing, which [OPS-rust-stable-behind-scripts](OPS-rust-stable-behind-scripts.md)
  leaves to a design revision that finds the mistakes call for it.

## Latitude

None.

## Alternatives considered

- **Leave it to review** — costs nothing to build. Three of five tickets show it costs a round of
  review each, and a reviewer finds an unrun branch only where it thinks to try a mutant.
- **Run it on instruction** — keeps `scripts/check-all.sh` as fast as it is. But
  [NUC-checks-are-bash §7](NUC-checks-are-bash.md) admits no such sensor, and a project ADR would
  extend a clause of the base for a run of some tens of seconds.
- **A step of CI alone** — a session would read it only after pushing, past the review it would
  have saved, and [NUC-checks-are-bash §5](NUC-checks-are-bash.md) keeps CI and the local gate
  alike.
- **A gate on a threshold** — a number to defend, and a threshold met by lines elsewhere hides the
  unrun lines of the change.
- **Mutation testing of the diff, as cargo-mutants `--in-diff`** — decides what coverage does not: a
  line a test runs and does not assert on, as the de-duplication of the citations in the work on
  `nuc upgrade`. It tests the suite once per mutant, minutes where coverage takes one test run. It
  stays for a design revision that finds the unrun lines reported and the undecided ones still
  costly.
- **`llvm-tools-preview` in `rust-toolchain.toml`** — one place for the toolchain, but every job of
  CI would install a component only the `sensors` job runs, against the toolchain of
  [OPS-rust-stable-behind-scripts](OPS-rust-stable-behind-scripts.md).
- **`LLVM_PROFILE_FILE` passed always** — simpler, but the fixtures' environment would widen in
  every build for a variable only a coverage build reads.
- **`cargo install --locked`** — no binary downloaded, but the container builds the tool from source
  at each creation. The maintainer chose the pinned release.
- **cargo-tarpaulin** — its default engine on Linux traces by ptrace, on x86_64 alone, which leaves
  an arm64 container out; its llvm engine reads the same instrumentation as cargo-llvm-cov.
- **grcov** — reads the same profiles, but leaves the instrumented build and the run of the tests
  to scripts of the project's own.

## Sources / Prior art

- cargo-llvm-cov — <https://github.com/taiki-e/cargo-llvm-cov>: installation, `llvm-tools-preview`,
  `cfg(coverage)`, the `lcov` output, the release archives and the attestations since 0.8.5;
  release v0.9.2 of 2026-11-20.
- The Rust compiler's instrument-coverage —
  <https://doc.rust-lang.org/rustc/instrument-coverage.html>: `LLVM_PROFILE_FILE` and the default
  profile an instrumented binary writes in its current directory.
- cargo-mutants — <https://mutants.rs/>: mutation testing of a diff.
- The design revision of 2026-11-22, ticket *Decide whether a sensor reports the lines no test
  runs*, and the reviews of the tickets of `nuc-upgrade` and of that revision.
- `.devcontainer/install-shellcheck.sh`, the pattern of a pinned installer.

## Consequences

- Positive: a session sees the branches its change leaves unrun before it hands the change to a
  review, and writes the test then, which saves the rounds the reviews spent on them. The report
  is of the change alone, so the lines it names are the session's to judge.
- Negative / trade-offs: one more tool in the Dev Container and in the `sensors` job, pinned by
  checksum, and the component `llvm-tools-preview` in both. Each run of `scripts/check-all.sh` adds
  an instrumented test run, in a target of its own, a few minutes on its first build. The pre-push
  hook runs `scripts/check-all.sh --changed`, which skips no sensor, so each push pays the run too,
  and the `sensors` job takes longer. A line run by a test that asserts nothing
  counts as run: the sensor shows what no test reaches, not what no test decides. Lines reached only
  by the parity harness, which runs `nuc` outside `cargo test`, read as unrun. Outside the Dev
  Container, the sensor ends `scripts/check-all.sh` with exit 2 until the tool is installed by hand,
  and the pre-push hook refuses each push until then.
  [OPS-rust-stable-behind-scripts](OPS-rust-stable-behind-scripts.md) does not name this sensor
  among its sensors by kind.
- Follow-ups: the sensor, its self-test, the installer, and the fixtures' variable, as tickets;
  naming the sensor among the sensors by kind when the toolchain ADR is next superseded; whether
  the bash under `scripts/lib/` gets such a report once a tool is in place.

## Enforcement

`scripts/test-sensor-unrun-lines.sh` cites this ADR's identifier and NUC-checks-are-bash. The
self-test writes an `lcov` report and hands it to the sensor with `--lcov <file>`. The fixture is a
change with a line a test runs and one no test runs. It fails unless the sensor names the second
alone and exits 0. It also runs the sensor with two stand-ins of `cargo` on `PATH`: one that lacks
`llvm-cov`, and one whose test run fails. It fails unless each ends with exit 2. The installer's
self-test cites this ADR and fails on an archive whose SHA-256 is not the pinned one. A test of
`tests/support/` cites this ADR and fails unless the cleared environment of a build without
`cfg(coverage)` passes `PATH` and `SystemRoot` alone. The build with `cfg(coverage)` is decided by
the sensor's own run: a profile lost to a fixture fails the instrumented test run.
