# OPS-crates-pass-deny-and-audit: `nuc` may depend on a crate from crates.io that is MIT-licensed, passes `cargo deny` and `cargo audit`, and is taken with only the features it uses

- **Status:** 🟡 proposed
- **Date:** 2026-11-15
- **Deciders:** Markus Brigl
- **Applies to:** the `[dependencies]` of `Cargo.toml`, `Cargo.lock`, `deny.toml`, and `scripts/check-deps.sh`

## Context

Up to the port of template release v0.10.0, `nuc` depended on no crate: its readers of TOML,
JSON, SARIF, and the bash's patterns are written by hand, and the Context of several ADRs says so,
among them [API-command-line-keyed-findings](API-command-line-keyed-findings.md) and
[API-output-formats-carry-reason](API-output-formats-carry-reason.md). Template release v0.10.0
adds checks whose bash is a set of extended regular expressions run by awk: the five forms of
`docs/opens-no-issue` and the paths an *Applies to* names for `docs/adr-refs`. Written by hand,
each pattern becomes a matcher of its own, and a mistake in one goes unseen where no case of the
template's tables reaches it.

The supply chain is a threat the specification names.
[OPS-rust-stable-behind-scripts](OPS-rust-stable-behind-scripts.md) audits the dependencies through
`scripts/check-deps.sh`, on every pull request. It runs `cargo deny` against `deny.toml`, with its
licenses, sources, and bans, and `cargo audit` against `Cargo.lock`. `deny.toml` allows the MIT
license alone and crates.io as the one registry, and bans `git2` and `gix`, since git is reached
only through its CLI ([SEC-git-cleared-environment](SEC-git-cleared-environment.md)). The maintainer
allowed the `regex` crate for the patterns of v0.10.0 in the session that ported it.

## Decision

We will let `nuc` depend on a crate from crates.io that is licensed under MIT, alone or as an
alternative, that `cargo deny` and `cargo audit` pass, and that `Cargo.toml` takes with only the
features `nuc` uses.

1. **The first crate:** `regex`, with the feature `std` alone. It brings `regex-automata` and
   `regex-syntax`, and its byte API reads the bash's patterns with the ASCII classes.
   `Cargo.lock` also holds `aho-corasick` and `memchr`, which only its other features build, and
   which `cargo deny` and `cargo audit` check as well.
2. **A crate's place:** a crate serves a reader or a writer `nuc` would otherwise write by hand.
   Review refuses a crate that reaches git, the network, or a process; `deny.toml` bans `git2`
   and `gix` by name.

**Out of scope:** the tools of this repository, such as `cargo deny` and `cargo audit` themselves,
and the crates of the release tooling under `release/`.

## Alternatives considered

- **No crate, every pattern written by hand** — no supply chain at all, but each of the bash's
  patterns becomes a matcher of its own, and the template adds more with each release.
- **`regex-lite`** — fewer lines and no dependency of its own, but slower on long inputs and
  without the byte API that reads a line that is no UTF-8 as the bash does.
- **Crates vendored into the repository** — no registry at build time, but every update is a
  change of thousands of lines to review, and `cargo audit` still has to read them.
- **Any license `cargo deny` can read** — more crates to choose from, but `deny.toml` allows MIT
  alone, and a second license would need a decision of its own.

## Sources / Prior art

- `deny.toml` and `scripts/check-deps.sh` of this repository.
- The `regex` crate's documentation of its features and of `regex::bytes`.

## Consequences

- Positive: the bash's patterns are read by one engine, as the bash reads them by one, and the
  ported rules stay short. Every crate passes the same audit as before, on every pull request.
- Negative / trade-offs: the build fetches crates from crates.io, and an advisory against one of
  them fails `scripts/check-deps.sh` until it is updated. The Context of the ADRs that say `nuc`
  depends on no crate is out of date.
- Follow-ups: none.

## Enforcement

`scripts/check-deps.sh` cites this ADR with `Verifies: OPS-crates-pass-deny-and-audit`. It runs
`cargo deny`, which fails on a license other than MIT, another source, or a banned crate. It runs
`cargo audit`, which fails on an advisory. Review holds that a crate brings only the features
`nuc` uses.
