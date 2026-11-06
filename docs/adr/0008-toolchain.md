# ADR-0008: Rust stable pinned in `rust-toolchain.toml`, cargo's build, test, fmt, and clippy behind `scripts/check-build.sh` on every platform of Q-4 as required checks, and the dependency audit behind `scripts/check-deps.sh`

- **Status:** 🟢 accepted
- **Date:** 2026-11-05
- **Deciders:** Markus Brigl
- **Applies to:** `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-deps.sh`, their jobs in `.github/workflows/checks.yml`, the `portability` job among them, the required checks of `CONTRIBUTING.md`, the ruleset on `main`, the benchmark workflow, and the Build, Test & Run section of `README.md`

## Context

The specification fixes the language: Rust stable with the toolchain pinned (*Constraints*).
The supply chain asks for dependencies audited by `cargo deny` and `cargo audit` on every pull
request, every ignored advisory with its reason (Q-3); the integrity of the gates asks that
nothing passes when it could not decide (Q-1); hook latency is measured over 1,000 calls (Q-5).
The project is small: one maintainer, agents working the tickets
([ADR-0007](0007-harness.md)).

The catalogue entry *Toolchain* offers three forms. *The linter's own rules first* fits a
one-person tool, and the design revision adds what the mistakes call for. *One tool per kind*
adds a tool for each column of its table. *Mutation testing on a schedule* fits a quality goal
that asks for tests that decide, as Q-1 does, but there is no code to mutate yet.

`cargo deny check` and `cargo audit` both fetch the RustSec advisory database, so the script
that runs them needs the network. [ADR-0002](0002-decisions-verified-by-tests.md) has every check
in `scripts/check-all.sh`, `checks.yml`, and the required checks of `CONTRIBUTING.md` alike, so
the pre-push hook runs it too.

Q-4 asks that the full test suite pass in CI on Linux x64, Linux arm64, macOS, and Windows x64.
A platform job outside the required checks lets a red platform through whenever nobody waits for
it, although [`AGENTS.md` §5](../../AGENTS.md#5-quality-bar--definition-of-done) forbids merging
red in CI. A job with a matrix reports one check run per entry, `<job> (<values>)`, which the
required checks of `CONTRIBUTING.md` name that way (check 9 of `scripts/check-docs.sh`). The
runners are not the bottleneck: the three platform runs take under a minute each, while the
required `parity` job takes ten to fifteen. The repository is public, and GitHub's billing states
that standard GitHub-hosted runners are free there.

## Decision

We will build, test, and lint with cargo on Rust stable pinned in `rust-toolchain.toml`, run
through `scripts/check-build.sh`, and audit the dependencies through `scripts/check-deps.sh`, each
a job of `checks.yml` among the required checks and a `run` line of `scripts/check-all.sh`.

- **Platforms:** `scripts/check-build.sh` runs on Linux x64 as the `build` job, and on Linux
  arm64, macOS, and Windows as the `portability` job, its matrix `os` the runner labels
  `ubuntu-24.04-arm`, `macos-latest`, and `windows-latest`, `fail-fast` off, so that one red
  platform does not hide another. `CONTRIBUTING.md` names `portability (ubuntu-24.04-arm)`,
  `portability (macos-latest)`, and `portability (windows-latest)` between its `required-checks`
  markers, and the ruleset on `main` requires them, which `scripts/sensor-repo-settings.sh`
  compares. `scripts/check-all.sh` runs `scripts/check-build.sh` on the machine it runs on; the
  job adds no line there.
- **Toolchain:** one fixed stable version in `rust-toolchain.toml`, with the components `rustfmt`
  and `clippy`; a pull request moves it.
- **`scripts/check-build.sh`** runs, in this order: `cargo fmt --check`;
  `cargo clippy --all-targets --locked -- -D warnings`; `cargo build --locked`;
  `cargo test --locked`. The README names these as its Build, Test, and Lint commands.
- **`scripts/check-deps.sh`** runs `cargo deny check` against `deny.toml` and `cargo audit`. An
  ignored advisory carries its reason in the file that ignores it. Without the network the script
  fails; it never passes on a database it could not fetch (Q-1).
- **Sensors, by kind:**
  - *Behaviour:* the tests, tied to the decisions and criteria by `Verifies:` markers.
  - *Maintainability:* clippy's `too_many_lines` and `cognitive_complexity` raised to errors, and
    rustc's `dead_code` through `-D warnings`; `scripts/sensor-tests-kept.sh`, which knows
    `#[ignore]`; duplication: none yet.
  - *Architecture fitness:* `cargo deny`'s `bans`, and the structural test of the shape of the
    code, which the decision on the shape names.
- **Benchmark:** the measure of Q-5 runs in a workflow of its own on a schedule, not in
  `checks.yml`. It lands with the port of the hooks, in the change that adds
  `nuc hook reference-transaction`, the command it measures.

**Out of scope:** which platforms Q-4 names, a decision of the specification; a platform beyond
the runner images GitHub hosts; mutation testing, which a design revision adds when the mistakes
call for it; a duplication tool; the test layout and harness, a decision of its own.

## Alternatives considered

- **`check-deps.sh` in CI only, and weekly** — keeps the network out of the pre-push hook, but a
  job outside `scripts/check-all.sh` breaks the three lists ADR-0002 holds together, and a push
  would reach CI unaudited.
- **Skip the audit when the network is down** — a pass on a database never fetched is the silent
  pass Q-1 forbids.
- **Mutation testing with `cargo-mutants` on a schedule now** — there is no code to mutate; the
  design revision adds it once there is.
- **A duplication tool (jscpd)** — none yet; the design revision adds it if duplication shows up.
- **The Q-5 benchmark in `checks.yml`** — the runners' timing noise would gate every push on a
  number that moves without a change.
- **The platform jobs not required, recorded** — a workflow of their own that does not gate the
  merge. A red platform slips through whenever nobody waits for it; Q-4 would hold only by habit.
- **The platform jobs not required, reported** — the review binding sensor or the pull request
  template lists each platform's result for the human who merges. It reminds rather than
  decides, and a reminder is read or skipped like the result it repeats.
- **Required, the workflow kept apart** — a workflow of its own whose three check runs are
  required. Check 9 compares the required checks with the jobs of `checks.yml` alone, so the list
  would have to stand outside its markers, unchecked, or the template's check would have to change.

## Sources / Prior art

- The catalogue entry [*Toolchain*](../../.agents/skills/propose-adr/concepts/toolchain.md) and
  the Rust row of its table.
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) and
  [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit), both reading the
  [RustSec advisory database](https://rustsec.org/).
- [The rustup book, *Overrides*](https://rust-lang.github.io/rustup/overrides.html): the
  `rust-toolchain.toml` file.
- GitHub, *Troubleshooting required status checks*: a required check counts only from an eligible
  trigger such as `pull_request` or `push`, and a workflow skipped by a path filter leaves its
  check pending, so a required job runs on every pull request.
  https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/troubleshooting-required-status-checks
- GitHub, *About billing for GitHub Actions*: "The use of standard GitHub-hosted runners is free
  … in public repositories". The page does not name each platform in that sentence.
  https://docs.github.com/en/billing/managing-billing-for-your-products/managing-billing-for-github-actions/about-billing-for-github-actions
- The check runs of this repository: a matrix job reports one check run per entry, named
  `<job> (<values>)`, as `portability (macos-latest)`, where the three platform runs take under a
  minute each and `parity` ten to fifteen.

## Consequences

- Positive: one script per concern, run alike by the pre-push hook and CI; the toolchain is the
  same on every machine; the audit cannot pass unseen (Q-1, Q-3); Q-4 gates every merge into
  `main`, whoever waits for the platforms or does not.
- Negative / trade-offs: a push without the network fails until it has one, or is pushed with
  `--no-verify`, visibly; `cargo deny` and `cargo audit` overlap on advisories, which Q-3 accepts.
  A pull request waits for three more runners, though not longer than for `parity`. An outage of
  one runner image blocks every merge until it recovers, or until a maintainer
  bypasses the ruleset. No check compares the names of the check runs with the values of the
  matrix: a label renamed in the matrix but not in `CONTRIBUTING.md` leaves the ruleset waiting
  for a check run that never reports, and an entry dropped from the list runs unrequired.
- Follow-ups: the benchmark workflow and its runner; mutation testing and a duplication tool, when
  a design revision calls for them; whether a runner image that moves under its label, as
  `macos-latest` does, should be pinned to a version; whether check 9 should read the values of a
  matrix, a change of the template's check.

## Enforcement

The golden path's tests cite this ADR with a `Verifies:` marker, and it flips to accepted with
them. Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) fails while the build and
deps jobs, the `run` lines of `scripts/check-all.sh`, and the required checks of `CONTRIBUTING.md`
disagree, and when the list names the `portability` job, which has a matrix, by its bare name
instead of its check runs. `scripts/sensor-repo-settings.sh`, run on a maintainer's instruction,
reports a ruleset on `main` that differs from the list.

**Not mechanically decidable:** that the values and the number of the check runs the list names
match the matrix of the job, which check 9 leaves to the workflow; and that the ruleset on GitHub
requires them, which the settings sensor reports only when a maintainer runs it, since no test of
this repository can reach the ruleset.
