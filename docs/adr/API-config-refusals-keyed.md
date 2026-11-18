# API-config-refusals-keyed: Every refusal `nuc` reports while it reads `nuc.toml` ends in a key `[config/toml:<reason>]`

- **Status:** 🟡 proposed
- **Date:** 2026-11-17
- **Deciders:** Markus Brigl
- **Applies to:** the refusals of `src/config.rs` and of a `nuc.toml` that cannot be read at all, their message on the standard error of `nuc check` and `nuc hook`, the keys `config/toml:<reason>` in the list of keys kept beside the contract's test, and the tests that assert them

## Context

G-5 asks that every hook and check fail closed, naming how to install `nuc`, where `nuc` is older
than the project's `nuc.toml` requires. `nuc` does: it reads `minimum` first and refuses as too
old, and it refuses a `nuc.toml` that is missing for `nuc check`, unreadable, or turns on a rule it
does not know. Each refusal exits 2 and writes one line on the standard error.

[API-command-line-with-measure](API-command-line-with-measure.md) gives every finding a key and
says nothing of a key for a refusal. The shims end each refusal of their own in a key
([API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md)). Of the refusals a template
release can cause in a hook, those of `nuc.toml` are the only ones without one.

The template is to switch its hooks to the shims, and its self-test of the hooks is to assert each
refusal by its exit code and its key, never by its wording. A hook with a `nuc` older than the
minimum is one of the cases it asserts, and without a key it could assert only the wording, which
a release of `nuc` may change.

## Decision

We will end every refusal `nuc` reports while it reads `nuc.toml` in a space and a key
`[config/toml:<reason>]`, with the reasons `missing`, `unreadable`, `old`, and `unknown`.

- **The reasons:**
  - `missing`: `nuc check` finds no `nuc.toml` at the root of the repository;
  - `unreadable`: the file cannot be read as text, or it is not the subset of TOML that
    [API-command-line-with-measure](API-command-line-with-measure.md) decides;
  - `old`: its `minimum` is above the running version;
  - `unknown`: it turns on a rule this `nuc` does not know.
- **The same line:** the refusal stays one line on the standard error, its message unchanged
  before the key. The run exits 2, for `nuc check` and `nuc hook` alike.
- **The area:** `config/toml` is no rule. No `nuc.toml` can turn it off, `nuc explain` does not
  take it, and no JSON or SARIF output carries it, since a run that refuses writes none.
- **Stable:** the four keys stand in the list of keys beside the contract's test and stay there,
  as a key of a finding does. A reason is added with the cause it names.

**Out of scope:**

- other refusals with exit 2, such as an unreadable file of the tree other than `nuc.toml`, a
  repository git cannot read, or an unknown command;
- the wording of the messages before the key.

## Alternatives considered

- **No key; the template asserts the wording.** The template's self-test would break on every
  rewording of a message of `nuc`, and its normalization's map would have to spare these lines.
- **One key per key of the file, such as `config/minimum:old` and `config/rules:unknown`.** It
  names the line at fault, but the reasons already say which, and the shims' keys show one ID per
  source with several reasons.
- **Every refusal of `nuc` keyed.** It would key the refusals of I/O and git as well, whose
  causes have no fixed list yet. The refusals of `nuc.toml` are those a template release causes
  and a test asserts.
- **Supersede API-command-line-with-measure with a clause for the key.** It would restate a large
  ADR for one clause that contradicts none of it.

## Sources / Prior art

- [API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md): the keys of the shims' own
  refusals, `[shim/<hook>:<reason>]`, the form this one follows.
- The template's plan of its hooks' switch, `docs/plan/hooks-run-nuc/` in `hivevm/nuc`, whose
  self-test asserts a hook with a `nuc` too old by its key.

## Consequences

- Positive: every refusal a template release can cause in a hook, of the shim and of `nuc.toml`,
  ends in a key, so a test of the hooks asserts none of them by its wording.
- Negative: the refusals of I/O and of git, in a hook and in a check, stay without a key.
- Positive: a reader of a refusal tells a too old `nuc` from an unreadable file without parsing
  the message.
- Negative: four more keys that stay once listed.
- Follow-ups: keys for the other refusals with exit 2, once a test needs them.

## Enforcement

- `tests/contract.rs` holds that each of the four refusals exits 2 and ends in its key on the
  standard error. A `nuc.toml` that is no UTF-8 text or no file refuses as unreadable. The keys
  stand in `tests/contract.rule-keys`. It cites this ADR.
- `tests/hooks.rs` holds that a hook with a `nuc` older than the minimum refuses in
  `[config/toml:old]`. It cites this ADR.
