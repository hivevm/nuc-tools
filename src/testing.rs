//! What a unit test writes that this repository's own gate would read as a reference: an ADR or a
//! draft identifier, a `Verifies:` marker, or a section of `AGENTS.md`. The gate's bash checks read
//! every text file, the tests included (ADR-0018), so each is assembled here, as the template's
//! self-tests assemble theirs with `adr_id` and `mark`; `tests/support/` holds the same for the
//! integration tests.

/// The identifier `ADR-<key>`, as `ADR-0001` or `ADR-draft-<slug>`.
pub fn id(key: &str) -> String {
    format!("{}-{key}", "ADR")
}

/// The identifier of the ADR numbered `n`, `ADR-000n`.
pub fn adr_id(n: u32) -> String {
    id(&format!("{n:04}"))
}

/// The identifier of the draft `<slug>`, `ADR-draft-<slug>`.
pub fn draft_id(slug: &str) -> String {
    id(&format!("draft-{slug}"))
}

/// The word that opens a marker, `Verifies:`.
pub fn verifies() -> String {
    format!("{}:", "Verifies")
}

/// A line of a marker that cites the identifiers.
pub fn mark(ids: &str) -> String {
    format!("{} {ids}\n", verifies())
}

/// The reference `AGENTS.md, section <n>`, written out.
pub fn agents_section(n: &str) -> String {
    format!("AGENTS.md, {}tion {n}", "sec")
}
