# Ticket: The Measures line takes its tokens from nuc measure

## What to build

`nuc measure` has been released since v0.2.0 (#100), yet `CONTRIBUTING.md` still says the tokens
of a hand-off's `Measures:` line come from OpenTelemetry "or, once it lands, from `nuc measure` of
hivevm/nuc-tools#81". The template dropped "once it lands" in v0.12.0; this project kept the older
sentence when it took that release up. The evidence is the cost data: 9 of the 10 hand-offs from
#100 to #109 carry `tokens -`, among them #105, #108, and #109, which a session wrote. So step 6
of this design revision had no tokens to read for most of the work in scope.

Name `nuc measure` as the way to get the tokens, with the command a hand-off runs, for the current
branch and for a `feat/` branch that gathers its tickets' branches. Say what `-` means after it:
the work ran in no harness `nuc measure` reads. The README's own section on `nuc measure` is the
reference; link it rather than restating it.

## Acceptance criteria

- [ ] Given `CONTRIBUTING.md`, when a session reads how to fill the `Measures:` line, then it
  names `nuc measure` and the command for a branch and for a `feat/` branch, and no longer
  speaks of a command still to land.
- [ ] Given the change, when `scripts/check-docs.sh` runs, then it passes.

## Read first

- Touches: `CONTRIBUTING.md`
- `.github/PULL_REQUEST_TEMPLATE.md` (the form of the line); the `nuc measure` section of
  `README.md`; API-measure-reads-harness-transcripts; the convention *A rule changes everywhere it
  is stated* in `docs/CONVENTIONS.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes what every hand-off is asked to report, and the human may prefer the
template's wording as it stands.
