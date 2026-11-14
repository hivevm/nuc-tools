# nuc ports the rules of template releases v0.9.0 and later

Observed: `nuc` places releases up to v0.8.x. Releases v0.9.0 to v0.11.0 name ADRs by category and
slug, hold a fixed base by `docs/adr/NUC.blobs`, and key each check by its rule. They add the
rules `docs/adr-identifiers`, `docs/base-blobs`, `shell/root`, `shell/bash-floor`, and `shell/mode`.
They change the hooks to integrate `feat/` branches linearly. A derived project on one of those
releases gets no `nuc check` that holds G-1 and Q-2. The corpus has no repository on them either.

Since v0.9.1 the template's fixture tables expect keys and data, never wording, so a port can be
held to the same cases.

Where: `src/rules/`, `src/shell/hook.rs`, `tests/parity/corpus.toml`, the fixture tables under
`scripts/fixtures/` of the template from v0.9.1 on.
