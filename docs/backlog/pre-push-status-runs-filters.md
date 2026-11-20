# nuc hook pre-push asks git status, which runs the repository's filters and submodules' status

Observed: `nuc hook pre-push` tells a clean working tree with `git status --porcelain`, to decide
whether the tree's green record holds. `git status` hashes each tracked file whose stat changed
through the filter its attributes name, running the driver's `clean` or `process` command, and it
asks each checked-out submodule for its own status, which runs that submodule's filters in turn
(Q-7). `SEC-git-cleared-without-lazy-fetch` neutralizes no filter key, so a driver of the
repository's or a submodule's configuration runs on every push. The call is older than the branch
that ports the sensors; the review of ticket 05 found it, and ticket 05 does not widen to it.

A fix needs: the status asked with every active filter driver neutralized, as
`neutralized_filters` of `src/shell/sense.rs` does for the sensors, and without looking into a
submodule's working tree; and a decision for what then cannot be told as the bash hook tells it. A
path under a filter whose raw bytes differ from the index, and a checked-out submodule, are clean or
not by commands nuc must not run: the hook can treat such a tree as not clean, which runs the
checks where the bash hook might skip them, or refuse. The parity scenarios of the hooks hold the
bash hook's answer, and a scenario with a filter and a submodule decides the choice.

Where: `src/shell/hook.rs` (`pre_push`, the `git status --porcelain` call), `src/shell/git.rs`
(`NEUTRALIZED`), the hook scenarios of `tests/parity/hook-scenarios.sh`, and `tests/hooks.rs`.
