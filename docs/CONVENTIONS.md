# Conventions — nuc-tools

> How this project writes code and tests **where no check decides it**: the judgement calls. What
> a module may know, what a test asserts and never does, the shape of an error, how a thing is
> named when the glossary has no word for it. Whoever settles a convention updates this file
> inline, an agent included, in the same change as the review or code that settled it. The
> reviewer always has it ([NUC-documents-are-fixed §6](adr/NUC-documents-are-fixed.md)).

An entry says in a sentence or two what is done and why; the why tells a convention from a habit.
A convention a check can decide is the check, not an entry. One that constrains future choices is
an ADR. What the golden path already shows needs none. Few entries that hold beat many that are
skimmed. The design revision drops what prevented nothing
([`AGENTS.md` §5](../AGENTS.md#5-quality-bar--definition-of-done)).

- **A commit that restates accepted ADRs carries its map** — a normalization, and a take-up of a
  template release that moves the citations of a `NUC-` clause in accepted ADRs as the release's
  map says, writes in its commit message a line that reads `ADR map:` alone, the map below it.
  `scripts/check-adr-text.sh` reads that line, and fails any other commit that changes an accepted
  ADR beyond its Status and Date lines; the `normalize` skill asks for the map in the body, not for
  the line.
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
  [ARCH-rules-outcome-hooks-answer](adr/ARCH-rules-outcome-hooks-answer.md),
  where a read may lead in Q-12, and where in the shell a read may stand in
  `tests/functional_core.rs`.
- **A port copies the bash, quirks included** — a rule reports what its bash check reports on
  inputs the corpus lacks as well, its exemptions and oddities with it: only the first Template
  release line counts, and a missing README is the finding of another rule. How its unit tests
  copy the cases of the bash self-test is stated in `tests/bash_cases.rs`. Where the bash result
  depends on more than the input, as grep's on a file with a NUL byte, `nuc` refuses instead of
  guessing. The author of a port, a hook's included, must try it on the bytes the bash reads
  otherwise than Rust, in a file, in what git answers, and in what git hands a hook on its standard
  input and arguments:
  - a byte of no valid UTF-8;
  - the byte 0x1F its awk passes split fields on;
  - a last line without a newline, which `read` drops and awk and `sed` keep;
  - numbers in bash arithmetic: octal after a leading zero, and 64-bit wrap;
  - a path git lists that is no UTF-8, which the bash opens and Rust's strings do not hold;
  - a path git writes in double quotes, as it writes one with a `"`, a `\`, or a control byte,
    which `adr::awk_unescape` reads as awk does;
  - a ref name or a message that is no UTF-8, which the bash reads as bytes.

  The author of a port must make it read only the files and the answers of git its bash reads, so
  that a file the bash never opens cannot refuse a run. The author must try each byte class that
  `nuc` reads as the bash does in the first place that can hold it:
  - a variant of the parity harness, where a variant can write it and the bash reports it without
    an error of its own;
  - for a hook, a scenario of the hook harness;
  - for a sensor, a scenario of `tests/parity/sensor-scenarios.sh`;
  - else a fixture of `tests/check.rs`, for a hook of `tests/hooks.rs`, for a sensor of
    `tests/contract.rs`;
  - else a unit test, as for a digit that bash's arithmetic lacks the base for.

  The author must pin each byte class that `nuc` refuses by a fixture of `tests/check.rs`, for a
  hook of `tests/hooks.rs`, for a sensor of `tests/contract.rs`, that shows exit 2.
  Which readings of the bash's tools a port takes from `src/bash.rs` is stated in
  `tests/functional_core.rs`.
- **A git command a port adds is tried against the repository's own commands** — the author of a
  port must test each git command the port adds against a repository whose configuration names a
  command git could run, or that makes git look into a submodule:
  - a clean filter and a process filter;
  - a textconv driver;
  - an external diff;
  - `diff.submodule`;
  - a checked-out submodule.

  The test must run the command in every form the port calls it, a range of commits and the working
  tree alike. The list of neutralized keys guards only a key whose name is fixed
  (SEC-git-cleared-without-lazy-fetch). A driver's name changes from repository to repository, so
  only such a test sees a command run under it (Q-7).
- **A check's cases are a table** — a new or changed check writes its cases into a table under
  `scripts/fixtures/` before its code, and its self-test reads the table, so a port of the
  checks is held to the same cases. A case expects a key, a rule identifier and a reason word that
  the message ends in, such as `[docs/adr-index:twice]`, and the data of the finding, never its
  wording, so that rewording a message breaks no port.
