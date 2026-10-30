# ADR-draft-toolchain: Rust stable pinned in `rust-toolchain.toml`, cargo's build, test, fmt, and clippy behind `scripts/check-build.sh`, and the dependency audit behind `scripts/check-deps.sh`

- **Status:** 🟡 proposed
- **Date:** 2026-10-30
- **Deciders:** Markus Brigl
- **Applies to:** `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-deps.sh`, their jobs in `.github/workflows/checks.yml`, the benchmark workflow, and the Build, Test & Run section of `README.md`

## Context

The specification fixes the language: Rust stable with the toolchain pinned (*Constraints*).
The supply chain asks for dependencies audited by `cargo deny` and `cargo audit` on every pull
request, every ignored advisory with its reason (Q-3); the integrity of the gates asks that
nothing passes when it could not decide (Q-1); hook latency is measured over 1,000 calls (Q-5).
The project is small: one maintainer, agents working the tickets
([ADR-draft-harness](draft-harness.md)).

The catalogue entry *Toolchain* offers three forms. *The linter's own rules first* fits a
one-person tool, and the design revision adds what the mistakes call for. *One tool per kind*
adds a tool for each column of its table. *Mutation testing on a schedule* fits a quality goal
that asks for tests that decide, as Q-1 does, but there is no code to mutate yet.

`cargo deny check` and `cargo audit` both fetch the RustSec advisory database, so the script
that runs them needs the network. [ADR-0002](0002-decisions-verified-by-tests.md) has every check
in `scripts/check-all.sh`, `checks.yml`, and the README's required checks alike, so the pre-push
hook runs it too.

## Decision

We will build, test, and lint with cargo on Rust stable pinned in `rust-toolchain.toml`, run
through `scripts/check-build.sh`, and audit the dependencies through `scripts/check-deps.sh`, each
a job of `checks.yml` among the required checks and a `run` line of `scripts/check-all.sh`.

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
  `checks.yml`.

**Out of scope:** the platform matrix of Q-4 and which of its jobs are required, decided with the
job in setup step 6; mutation testing, which a design revision adds when the mistakes call for
it; a duplication tool; the test layout and harness, a decision of its own.

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

## Sources / Prior art

- The catalogue entry [*Toolchain*](../../.agents/skills/propose-adr/concepts/toolchain.md) and
  the Rust row of its table.
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) and
  [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit), both reading the
  [RustSec advisory database](https://rustsec.org/).
- [The rustup book, *Overrides*](https://rust-lang.github.io/rustup/overrides.html): the
  `rust-toolchain.toml` file.

## Consequences

- Positive: one script per concern, run alike by the pre-push hook and CI; the toolchain is the
  same on every machine; the audit cannot pass unseen (Q-1, Q-3).
- Negative / trade-offs: a push without the network fails until it has one, or is pushed with
  `--no-verify`, visibly; `cargo deny` and `cargo audit` overlap on advisories, which Q-3 accepts.
- Follow-ups: the platform matrix and its required jobs; the benchmark workflow and its runner;
  mutation testing and a duplication tool, when a design revision calls for them.

## Enforcement

The golden path's tests cite this ADR with a `Verifies:` marker, and it flips to accepted with
them. Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) fails while the build and
deps jobs, the `run` lines of `scripts/check-all.sh`, and the README's required checks disagree.
