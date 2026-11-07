# Conventions — nuc-tools

> How this project writes code and tests **where no check decides it**: the judgement calls. What
> a module may know, what a test asserts and never does, the shape of an error, how a thing is
> named when the glossary has no word for it. Whoever settles a convention updates this file
> inline, an agent included, in the same change as the review or code that settled it. The
> reviewer always has it ([ADR-0001](adr/0001-agent-governance-model.md)).

An entry says in a sentence or two what is done and why; the why tells a convention from a habit.
A convention a check can decide is the check, not an entry. One that constrains future choices is
an ADR. What the golden path already shows needs none. Few entries that hold beat many that are
skimmed. The design revision drops what prevented nothing
([`AGENTS.md` §5](../AGENTS.md#5-quality-bar--definition-of-done)).

- **A rule changes everywhere it is stated** — a change to a rule, a step, or a name searches
  the tree for its old wording and corrects it in the same change. A link check sees an anchor,
  and `tests/adr_citations.rs` a citation of a superseded or rejected ADR; neither sees a sentence
  that restates the rule.
- **A self-test asserts what it builds** — it takes its fixtures from the tree it copies or writes
  them itself, never from the template's own ADRs, scripts, or numbers, so it holds in a derived
  project and after a normalization.
- **A test of an absence first shows the run it watches** — a test that asserts that nothing was
  run, printed, or written asserts first what the run did: its exit status, the exact answer it
  expects, the path past an early exit. Otherwise a run that never reached `nuc`, a binary
  missing from the `PATH` or a hook that left before the part under test, passes it.
- **The shell reads, the rules decide** — the shell reads every file a rule needs once, into the
  inputs, before any rule runs, and a rule gets text, never a path. A rule's tests are then cases
  of text without a repository, and a file `nuc` cannot read ends the run before a rule could
  pass on less than it should have seen (Q-1). Where a refusal stands is decided in
  [ADR-0024](adr/0024-rules-return-an-outcome-and-hooks-an-answer.md),
  where a read may lead in Q-12, and where in the shell a read may stand in
  `tests/functional_core.rs`.
- **A port copies the bash, quirks included** — a rule reports what its bash check reports on
  inputs the corpus lacks as well, its exemptions and oddities with it: only the first Template
  release line counts, and a missing README is the finding of another rule. How its unit tests
  copy the cases of the bash self-test is stated in `tests/bash_cases.rs`. Where the bash result
  depends on more than the input, as grep's on a file with a NUL byte, `nuc` refuses instead of
  guessing. Each port, a hook's included, is tried on the bytes the bash reads otherwise than
  Rust, in a file, in what git answers, and in what git hands a hook on its standard input and
  arguments: a byte of no valid UTF-8, the byte 0x1F its awk passes split fields on, a last line
  without a newline (`read` drops it, awk and `sed` keep it), numbers in bash arithmetic (octal
  after a leading zero, 64-bit wrap), a path git lists that is no UTF-8, which the bash opens and
  Rust's strings do not hold, and a ref name that is no UTF-8, which the bash reads as bytes. A
  byte class `nuc` reads as the bash does is tried by a variant of the parity harness where a
  variant can write it and the bash reports it without an error of its own, or for a hook by a
  scenario of the hook harness, else by a fixture of `tests/check.rs`, for a hook of
  `tests/hooks.rs`, or a unit test, as a digit bash's arithmetic lacks the base for; one it refuses
  is pinned by a fixture of `tests/check.rs`, for a hook of `tests/hooks.rs`, that shows exit 2.
  Which readings of the bash's tools a port takes from `src/bash.rs` is stated in
  `tests/functional_core.rs`.
