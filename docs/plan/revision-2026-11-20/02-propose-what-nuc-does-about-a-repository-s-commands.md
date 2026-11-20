# Ticket: Propose what nuc neutralizes or refuses of a repository's own commands

## What to build

SEC-git-cleared-without-lazy-fetch neutralizes a fixed list of configuration keys by `-c`. Under
it, "a key that makes a git command `nuc` uses run another command joins the list". That cannot
cover the commands of `nuc sense`, whose diffs meet commands named under keys that change from
repository to repository:

- a clean or process filter `filter.<driver>.*`;
- a textconv driver `diff.<driver>.textconv`;
- an external diff, both `diff.external` and `diff.<driver>.command`;
- `diff.submodule`, which starts a diff inside the submodule;
- a checked-out submodule.

The code now neutralizes each active filter driver by name. Where neutralizing could change what
the sensor reports, the sensor refuses. Each diff call adds `--no-ext-diff --no-textconv`, and
`git::NEUTRALIZED` carries `diff.submodule=short`. Four rounds of review worked this policy out
for ticket 06 of `sensors-under-nuc-sense` (rounds 2 to 5), and no record decided it first:

- one refusal that blocked every repository with LFS;
- rename detection under a neutralized filter, which made `--require` pass where the bash fails;
- a C-quoted path under a filter that escaped the refusal;
- driver names that were exempted;
- drivers configured only in the environment, which `nuc` cannot see.

Ticket 05 met the same class twice: `--ignore-submodules=dirty`, and `diff.submodule`.
`docs/backlog/pre-push-status-runs-filters.md` records it in `nuc hook pre-push`.

Propose an ADR that decides this policy for every git call of `nuc`, and settle it with the
maintainer. It must decide:

- which commands `nuc` neutralizes, by key, by driver name, or by flag;
- where `nuc` refuses as not decided, and where it reads what the bash would read otherwise;
- what a driver configured only in the environment does to G-9.

It supersedes SEC-git-cleared-without-lazy-fetch or stands beside it.

## Acceptance criteria

- [ ] Given the policy the code holds today, when the session proposes the ADR, then its
  *Decision* names each command of the list above and what `nuc` does about it, and its *Context*
  cites the review rounds above.
- [ ] Given the ADR, when the maintainer accepts it, then the session moves the citations of the
  superseded ADR, if it supersedes one, in the change that flips it.

## Read first

- Touches: `docs/adr/`, `src/shell/git.rs`, `src/shell/sense.rs`
- SEC-git-cleared-without-lazy-fetch; API-git-only-sensors-under-sense;
  `docs/backlog/pre-push-status-runs-filters.md`. The reviews lived only in the sessions' untracked
  scratch files; the list above is the evidence.

## Blocked by

- None (can start immediately)

## Mode

with the human: it proposes an ADR on the code that spawns git, a protected part of
DEV-one-agent-one-ticket, and the maintainer alone decides ADRs.
