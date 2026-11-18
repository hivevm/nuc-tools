# NUC-hooks-run-nuc: The git hooks are the reference shims of `nuc`, pinned by their blobs, and refuse without the `nuc` that `nuc.toml` names

- **Status:** 🟢 accepted
- **Date:** 2026-11-17
- **Deciders:** NUC maintainer
- **Applies to:** the git hooks under `.githooks/` and the blob list beside them, `nuc.toml`, the installation of `nuc` in `.devcontainer/` and in the jobs of `.github/workflows/` that run a hook, `scripts/check-hooks.sh` and `scripts/lib/pin.sh`, the rule of `scripts/check-devcontainer.sh` on the nuc Feature, and the self-tests `scripts/test-git-hooks.sh` and `scripts/test-check-hooks.sh`

## Context

The template ships its git hooks as bash scripts under `.githooks/`, and every derived project
carries a copy. A fix to a hook reaches a project only when it takes up a template release and
merges the copy by hand. A hook runs before any check can say which bash it found, so
[NUC-checks-are-bash](NUC-checks-are-bash.md) holds each hook to bash 3.2, the one macOS ships.

`nuc`, the binary of `hivevm/nuc-tools`, ports the hooks of the template: `nuc hook <name>`
decides as the bash hook of the release a tree carries. `nuc-tools` releases reference shims in
POSIX sh that hand each hook to it. From v0.2.1 on, every refusal of a shim's own ends in a key
`[<rule>:<reason>]`; a refusal `nuc` reports while it reads `nuc.toml` does not yet. `nuc` reads
`nuc.toml`, which names the minimum `nuc` a tree requires and the rules it turns on; a `nuc` older
than the minimum refuses. The Feature
`ghcr.io/hivevm/features/nuc` installs `nuc` in a Dev Container, and the Action of `nuc-tools`
installs it in CI. Both verify the binary against the checksums of its release.

A derived project cannot switch alone. `nuc-tools` tried, under
[NUC-checks-are-bash §8](NUC-checks-are-bash.md): the template's self-test of the hooks holds their
wording, the self-test of the normalize skill expects the hooks' citations to follow a map, which
the compiled messages of `nuc` cannot, and the check of the Dev Container asks for `.githooks` as
the hooks path. These files change with the hooks, so the switch happens in the template.

Three more forces bound the answer. A hook that passes when it cannot decide is a gate nobody has.
A shim edited by hand, or left behind a `nuc` release, decides something no release of `nuc`
tested. A test that holds a message's wording breaks on every rewording and on every map of a
normalization, while a key stays.

## Decision

We will make the template's git hooks the reference shims of `nuc`, pinned by their blobs, and have
each hook refuse where `nuc` is missing or older than `nuc.toml` names.

1. **Shims.** Each of the five hooks under `.githooks/`, `pre-commit`, `pre-merge-commit`,
   `pre-push`, `reference-transaction`, and `merge-adr-index`, is the file of the same name under
   `shims/` of a release of `hivevm/nuc-tools` that ends every refusal of a hook in a key, those of
   `nuc.toml` included, unchanged. A hook itself must need no more than a POSIX sh, git, and `nuc`
   on the `PATH`; the checks it runs need what [NUC-checks-are-bash §1](NUC-checks-are-bash.md)
   names. It hands its work to `nuc hook <name>`, which refuses and passes as the table of the git
   hooks in `CONTRIBUTING.md` says, and runs the checks as
   [NUC-checks-are-bash §6](NUC-checks-are-bash.md) decides.
2. **Pin.** A blob list beside the hooks names each of the five hooks' blobs and the release of
   `nuc-tools` it comes from. A check fails where one of the listed hooks differs from its blob. A
   newer release of the shims replaces the hooks and the list in one change.
3. **`nuc.toml`.** The template ships a `nuc.toml` at the root of the tree. Its `minimum` is at
   least the release the shims come from. Its `rules` name every `hook/` rule of that `nuc`, since
   with a `nuc.toml` its list governs which rules run. A project must not remove a `hook/` rule.
4. **Installation.** The Dev Container installs `nuc` by the Feature `ghcr.io/hivevm/features/nuc`,
   referenced and locked as
   [NUC-container-hides-host-daemon §3](NUC-container-hides-host-daemon.md) says. Each job of CI
   that runs a hook installs `nuc` before it, from a release, verified against the checksums of
   that release.
5. **Refusal.** A hook without `nuc`, with a `nuc` older than the minimum, or with an answer of
   `nuc` it does not know, must refuse and end its message in its key. No hook falls back to bash.
6. **Self-test.** `scripts/test-git-hooks.sh` must assert each refusal by its exit code and its key,
   never by its wording, with a released `nuc` no older than the `minimum` of `nuc.toml` on the
   `PATH`.

**Out of scope:**

- the checks and sensors, which stay bash scripts under
  [NUC-checks-are-bash](NUC-checks-are-bash.md), and the rules of `nuc check` that would replace
  them;
- an update of `nuc` by `nuc` itself;
- how `nuc-tools` builds, tests, and releases `nuc` and its shims;
- hooks a project adds of its own.

## Latitude

A project may choose:

- further rules in `nuc.toml`. A rule that replaces a script of the template needs an ADR that
  narrows [NUC-checks-are-bash §1](NUC-checks-are-bash.md), under its clause 8;
- how a contributor outside the Dev Container installs `nuc`, which needs no ADR;
- how CI installs `nuc` under clause 4. The template's workflow uses the Action of `nuc-tools`
  by its major tag, which installs the release that tag names and verifies it. A project chooses
  how its workflows reference actions from the entry
  [Action references](../../.agents/skills/propose-adr/concepts/action-references.md) of the
  catalogue. A project that pins actions by commit SHA installs `nuc` by the installer of a
  release instead, with its checksums committed: a tool that moves the pin of the Action moves it
  to the source commit of a release, which carries no checksums and installs nothing.

## Alternatives considered

- **Keep the bash hooks.** A fix to a hook keeps reaching a project only by a merged copy, and a
  hook keeps needing bash and its tools.
- **Each project switches alone, under NUC-checks-are-bash §8.** `nuc-tools` tried it. The
  template's own self-tests and the check of the Dev Container hold the bash hooks, so a project
  would carry changed template files that every take-up merges again.
- **Move every check to `nuc` at once.** It removes more bash, but the checks behind `nuc check`
  are a larger change with its own parity to prove. The hooks go first, and the checks follow in
  a change of their own.
- **Fall back to the bash hooks where `nuc` is missing.** Two implementations decide one gate, and
  the fallback keeps every cost the switch removes. A contributor without `nuc` never learns that
  the hooks run on an old copy.
- **A hook that downloads `nuc` on its first run.** It puts the network into every hook and
  refuses work whenever the network fails. The Feature and the Action already install a verified
  binary.
- **Copy the shims without a pin.** A shim edited by hand would decide unseen, and a shim left
  behind a release would stay so until someone diffed it.

## Sources / Prior art

- Git, *githooks* — <https://git-scm.com/docs/githooks>: hooks are found through `core.hooksPath`
  and run as executables, in any language.
- `hivevm/nuc-tools`, the section *Git hooks* of its README and its ADR of the keyed shims —
  <https://github.com/hivevm/nuc-tools>: the shims, their answers, and their keys.
- Dev Container Features — <https://containers.dev/features>: a Feature installs a tool at build
  time, locked by its digest.

## Consequences

- Positive: a fix to a hook ships once, in `nuc`, and reaches every project with its next `nuc`.
- Positive: a hook itself needs a POSIX sh, not bash 3.2.
- Negative: the checks a hook runs still need bash 4.4, so a contributor outside the Dev Container
  still installs a current bash.
- Positive: the self-test of the hooks asserts keys, so rewording a message breaks no test, and a
  normalization's map runs over the tree.
- Negative: a contributor installs `nuc` before the first commit; outside the Dev Container, by
  hand.
- Negative: every project's local gate depends on a release of `nuc-tools`. The checksums of the
  release and the lock of the Feature bound what reaches it.
- Negative: a change to the shims themselves still reaches a project only by a template release.
- Negative: the shims cite clauses of `NUC-` ADRs, and those citations follow a release of
  `nuc-tools`, not a normalization: the checks and the map script skip the pinned files. A
  release that moves such a clause leaves the citation stale until the next shims.
- Follow-ups: the correction in place of clause 3 of NUC-checks-are-bash, its scope, and its
  latitude, which lands with the switch of the hooks; the checks behind `nuc check`; an update of
  `nuc` by itself.

## Enforcement

- `scripts/test-git-hooks.sh` runs every row of the table of the git hooks on fixture repositories
  with the hooks under `.githooks/`. It asserts each refusal by its exit code and its key, among
  them a hook without `nuc` and one with a `nuc` older than the minimum (clauses 1, 5, and 6). It
  cites this ADR.
- A check of the hooks' blobs against the blob list fails where a listed hook is not the shim it
  names (clause 2), and where `nuc.toml` names a `minimum` below the release of the shims or lacks
  a `hook/` rule of that release (clause 3). A self-test with a fixture for each cites this ADR.
- `scripts/check-devcontainer.sh` fails where a Dev Container definition lacks the Feature
  `ghcr.io/hivevm/features/nuc`, and holds its tag and lock as NUC-container-hides-host-daemon
  decides (clause 4).

**Not mechanically decidable:** whether each job of CI that runs a hook installs `nuc` before it
(clause 4); the review holds it.
