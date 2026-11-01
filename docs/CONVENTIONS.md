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
  the tree for its old wording and for every citation of its anchor, and corrects them in the
  same change. A link check sees the anchor, never a sentence that restates it.
- **A self-test asserts what it builds** — it takes its fixtures from the tree it copies or writes
  them itself, never from the template's own ADRs, scripts, or numbers, so it holds in a derived
  project and after a normalization.
- **The shell reads, the rules decide** — the shell reads every file a rule needs once, into the
  inputs, before any rule runs, and a rule gets text, never a path. A rule's tests are then cases
  of text without a repository, and a file `nuc` cannot read ends the run before a rule could
  pass on less than it should have seen (Q-1). What one rule cannot decide of what it read is the
  refusal of its outcome, beside the rule, never a case in the shell; what several rules read
  alike is refused where the shell reads it. Every file the shell reads and every directory it
  lists resolves inside the repository, or the run is not decided, so that no link, symbolic link,
  or absolute path in the tree makes `nuc` read what the tree does not hold; each new reading goes
  through the shell's `Tree`, which tests that, and the structural test fails on a read, a
  listing, or a resolution in any other function of the shell. A test of existence, as the
  bash's `-e`, `-f`, and `-d`, follows the path wherever it leads and reads nothing.
- **A port copies the bash, quirks included** — a rule reports what its bash check reports on
  inputs the corpus lacks as well, its exemptions and oddities with it: only the first Template
  release line counts, and a missing README is the finding of another rule. Its unit tests copy
  the cases of the bash self-test. Where the bash result depends on more than the input, as grep's
  on a file with a NUL byte, `nuc` refuses instead of guessing. Each port is tried on the bytes
  the bash reads otherwise than Rust: a byte of no valid UTF-8, the byte 0x1F its awk passes split
  fields on, a last line without a newline (`read` drops it, awk and `sed` keep it), numbers in
  bash arithmetic (octal after a leading zero, 64-bit wrap), and a path git lists that is no
  UTF-8, which the bash opens and Rust's strings do not hold. A byte class `nuc` reads as the bash
  does is tried by a variant of the parity harness where a variant can write it and the bash
  reports it without an error of its own, else by a fixture of `tests/check.rs` or a unit test, as
  a digit bash's arithmetic lacks the base for; one it refuses is pinned by a fixture of
  `tests/check.rs` that shows exit 2. A port reads lines and POSIX space
  through `src/bash.rs`, never through a copy of its own, and the structural test fails on a
  reader of `tests/functional_core.bash` defined elsewhere.
