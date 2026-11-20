# nuc sense binding-adrs reads a checked-out submodule under its own neutralized filters

Observed: in a change that ends in the working tree, `nuc sense binding-adrs` refuses every
checked-out submodule with `[sense/binding-adrs:submodule]`, even a clean, unchanged one. The
bash's git may count a submodule as changed by its own status, and that status runs the
submodule's filters (Q-7), so `nuc` does not ask for it. The run with no argument therefore exits 2
on every repository with a checked-out submodule; `--path` and a given head still report. The
maintainer chose the refusal for ticket 05.

The extension: read the submodule's state with git in the submodule, under the cleared environment
and its own filter drivers neutralized, as `neutralized_filters` does for the superproject:
`diff --name-only` and `ls-files --others --exclude-standard`. Refuse only where a path there under
a filter differs from its index by its raw bytes. Follow nested submodules. Count the submodule as
the effective `ignore` setting of `.gitmodules`, the configuration, and `diff.ignoreSubmodules`
has the bash's git count it.

Where: `src/shell/sense.rs` (`refuse_checked_out_submodule`, `changed_paths`), the contract test
`binding_adrs_in_the_working_tree_runs_no_filter` in `tests/contract.rs`, and the parity scenarios
of `tests/parity/sensor-scenarios.sh`.
