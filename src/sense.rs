//! The sensors `nuc sense` ports from the template (API-sensor-cores-ask-the-shell): each a
//! function of the values the shell gathered, or of the answers to a trait only the shell
//! implements where its bash reads by each answer, which returns its report, the bytes the bash
//! sensor of the tree's release writes on the standard output, or a refusal. A sensor reads no file
//! and runs no process; the shell reads, prints, and exits (API-command-line-with-sense).

pub mod action_refs;
pub mod backlog;
pub mod binding_adrs;
pub mod change;
pub mod plain_rules;
pub mod plan_frontier;
pub mod review_binding;
pub mod revision_due;
pub mod tests_kept;

use std::collections::BTreeSet;
use std::fmt;

use crate::config::Version;
use crate::rules::{self, Carried};

/// A sensor `nuc sense` knows: its name, the part of the script's name after `sensor-` and before
/// `.sh`; the earliest release whose bash sensor it reports as, before which the template has none
/// or one that reported otherwise; the releases from which it reports as their
/// bash does and the one before did not, which `tests/parity/releases.tsv` holds to the template's
/// trees; and the reasons of its refusals, each the end of a key `sense/<name>:<reason>`.
pub struct Sensor {
    pub name: &'static str,
    pub since: Version,
    pub changes: &'static [Version],
    pub reasons: &'static [&'static str],
}

/// Every sensor `nuc sense` knows. The template's sensors that need a process other than git, or
/// the network, are none of them (API-sensor-cores-ask-the-shell).
pub const ALL: &[Sensor] = &[
    action_refs::SENSOR,
    backlog::SENSOR,
    binding_adrs::SENSOR,
    plain_rules::SENSOR,
    plan_frontier::SENSOR,
    review_binding::SENSOR,
    revision_due::SENSOR,
    tests_kept::SENSOR,
];

/// The key of a name `nuc sense` does not know.
pub const UNKNOWN: &str = "sense/unknown:name";

/// The reason every sensor refuses a tree whose release has no such sensor with.
pub const RELEASE: &str = "release";

/// A sensor that cannot decide the tree: the reason its key ends in, and why.
#[derive(Debug, PartialEq, Eq)]
pub struct Refusal {
    pub sensor: &'static str,
    pub reason: &'static str,
    pub message: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} — not decided [sense/{}:{}]",
            self.message, self.sensor, self.reason
        )
    }
}

/// The sensor of that name, if `nuc sense` knows it.
pub fn find(name: &str) -> Option<&'static Sensor> {
    ALL.iter().find(|sensor| sensor.name == name)
}

/// The release a sensor reports as: the one the README names, or the newest `nuc` knows where it
/// names none or `unreleased`; refused where that release has no such sensor.
pub fn release(sensor: &Sensor, carried: Option<Carried>) -> Result<Version, Refusal> {
    let newest = *rules::RELEASES.last().expect("nuc knows a release");
    let release = match carried {
        Some(Carried::Release(release)) => release,
        Some(Carried::Unreleased) | None => newest,
    };
    if release < sensor.since {
        return Err(Refusal {
            sensor: sensor.name,
            reason: RELEASE,
            message: format!(
                "the tree carries template release v{release}, before v{}, the earliest whose \
                 sensor-{}.sh nuc reports as",
                sensor.since, sensor.name
            ),
        });
    }
    Ok(release)
}

/// The positional arguments a sensor reads, at most `most` of those left once it read its own
/// options, or none where `nuc sense` refuses them. One rule holds for every sensor that takes a
/// base and a head:
/// - an argument that reads as an option is refused on every release, wherever it stands, as the
///   bash of v0.10.0 on refuses it, where an earlier bash takes it as a base or a head, which names
///   no commit, or ignores it past them;
/// - an argument past `most` is refused where `refuses_extra`, from the release whose bash refuses
///   it, and ignored before, as the bash before ignores it (API-sensor-cores-ask-the-shell).
pub fn positional<'a, 'b>(
    rest: &'b [&'a str],
    most: usize,
    refuses_extra: bool,
) -> Option<&'b [&'a str]> {
    if rest.iter().any(|arg| arg.starts_with('-')) || (refuses_extra && rest.len() > most) {
        return None;
    }
    Some(&rest[..rest.len().min(most)])
}

/// The reason every sensor refuses with where the caller's environment names a command git would
/// run on a diff, a filtered file, or a hash of the working tree, which `nuc`'s git does not see
/// (SEC-repository-commands-neutralized-or-refused).
pub const ENVIRONMENT: &str = "environment";

/// The reason of a sensor that meets a hunk's head whose number, or awk's sum of its numbers, runs
/// past 2^53, which awk's double does not hold exactly: git writes none, and nuc does not decide
/// one.
pub const HUNK: &str = "hunk";

/// The refusal of the sensor for a hunk's head whose number awk reads inexactly.
pub fn hunk_refusal(sensor: &'static str) -> impl Fn(crate::bash::Overflow) -> Refusal {
    move |_| Refusal {
        sensor,
        reason: HUNK,
        message: "a hunk's head whose number, or a sum of its numbers, runs past 2^53, which awk \
                  reads inexactly"
            .to_owned(),
    }
}

/// The reason of a sensor that refuses a path whose diff the bash's git reads through a driver of
/// the repository (SEC-repository-commands-neutralized-or-refused).
pub const DRIVER: &str = "driver";

/// What of the configuration changes what the bash's porcelain diff compares, where `nuc`'s,
/// under `--no-textconv`, `--no-ext-diff`, and `diff.submodule=short`, compares the raw bytes.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DiffDrivers {
    /// The drivers any entry gives a `textconv`.
    pub textconv: BTreeSet<Vec<u8>>,
    /// The drivers any entry gives a `command`.
    pub command: BTreeSet<Vec<u8>>,
    /// Whether an entry sets `diff.external`.
    pub external: bool,
    /// Whether an entry sets `diff.submodule` to `diff` or `log`, which prints a submodule's hunks.
    pub submodule: bool,
}

impl DiffDrivers {
    /// Whether a path's hunks can read otherwise than its raw bytes: a driver is active, or
    /// `diff.external` set.
    pub fn any(&self) -> bool {
        self.external || !self.textconv.is_empty() || !self.command.is_empty()
    }
}

/// The diff drivers of the configuration, from the output of
/// `git config -z --get-regexp '^diff\.'`, `<key> LF <value> NUL` an entry. The
/// `diff.submodule=short` that `nuc` sets itself with `-c` is among them, and reads as no driver.
/// A command set empty counts too: git runs it and fails, and the bash can report as though the
/// diff were empty, so its report is not decided either.
pub fn diff_drivers(config: &[u8]) -> DiffDrivers {
    let mut drivers = DiffDrivers::default();
    for entry in config.split(|&b| b == 0).filter(|e| !e.is_empty()) {
        let (key, value) = match entry.iter().position(|&b| b == b'\n') {
            Some(at) => (&entry[..at], &entry[at + 1..]),
            None => (entry, &b""[..]),
        };
        let Some(rest) = key.strip_prefix(b"diff.") else {
            continue;
        };
        match rest {
            b"external" => drivers.external = true,
            b"submodule" => drivers.submodule |= value == b"diff" || value == b"log",
            _ => {
                // `diff.<name>.<variable>`, the name between the first and the last dot.
                let Some(dot) = rest.iter().rposition(|&b| b == b'.') else {
                    continue;
                };
                let (name, variable) = (&rest[..dot], &rest[dot + 1..]);
                match variable {
                    b"textconv" => drivers.textconv.insert(name.to_vec()),
                    b"command" => drivers.command.insert(name.to_vec()),
                    _ => false,
                };
            }
        }
    }
    drivers
}

/// The first submodule whose commit a patch of `git diff` moves, adds, or deletes, by the mode
/// `160000` of its header, named by its path on the new side, or the old where it is deleted.
pub fn moved_submodule(diff: &[u8]) -> Option<Vec<u8>> {
    let mut gitlink = false;
    let mut old: &[u8] = b"";
    for line in diff.split(|&b| b == b'\n') {
        if line.starts_with(b"diff --git ") {
            gitlink = false;
        } else if (line.starts_with(b"index ")
            || line.starts_with(b"new file mode ")
            || line.starts_with(b"deleted file mode ")
            || line.starts_with(b"old mode ")
            || line.starts_with(b"new mode "))
            && line.ends_with(b" 160000")
        {
            gitlink = true;
        } else if let Some(path) = line.strip_prefix(b"--- ") {
            old = path.strip_prefix(b"a/").unwrap_or(path);
        } else if let Some(path) = line.strip_prefix(b"+++ ")
            && gitlink
        {
            let new = path.strip_prefix(b"b/").unwrap_or(path);
            return Some(if new == b"/dev/null" { old } else { new }.to_vec());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_diff_drivers_are_the_repository_s() {
        let config = b"diff.bait.textconv\nconv\0diff.empty.textconv\n\0diff.cmd.command\nx\0diff.submodule\nshort\0diff.x.xfuncname\n^f\0";
        let drivers = diff_drivers(config);
        assert_eq!(
            drivers.textconv,
            BTreeSet::from([b"bait".to_vec(), b"empty".to_vec()])
        );
        assert_eq!(drivers.command, BTreeSet::from([b"cmd".to_vec()]));
        assert!(!drivers.external && !drivers.submodule);
        let drivers =
            diff_drivers(b"diff.submodule\nlog\0diff.external\n\0diff.submodule\nshort\0");
        assert!(drivers.external && drivers.submodule && drivers.any());
        assert!(!diff_drivers(b"diff.submodule\nshort\0").any());
    }

    #[test]
    fn a_moved_submodule_is_named_by_its_path() {
        let moved = b"diff --git a/sm b/sm\nindex 1111111..2222222 160000\n--- a/sm\n+++ b/sm\n@@ -1 +1 @@\n-Subproject commit 1\n+Subproject commit 2\n";
        assert_eq!(moved_submodule(moved), Some(b"sm".to_vec()));
        let deleted = b"diff --git a/f b/f\nindex 1..2 100644\n--- a/f\n+++ b/f\ndiff --git a/sm b/sm\ndeleted file mode 160000\nindex 1..0\n--- a/sm\n+++ /dev/null\n";
        assert_eq!(moved_submodule(deleted), Some(b"sm".to_vec()));
        assert_eq!(
            moved_submodule(
                b"diff --git a/f b/f\nindex 1..2 100644\n--- a/f\n+++ b/f\n+x 160000\n"
            ),
            None
        );
    }

    #[test]
    fn a_release_before_the_sensor_is_refused_and_none_reads_as_the_newest() {
        let sensor = &revision_due::SENSOR;
        let refusal = release(sensor, Some(Carried::Release(Version(0, 0, 9)))).unwrap_err();
        assert_eq!(refusal.reason, RELEASE);
        assert!(
            refusal
                .to_string()
                .ends_with("[sense/revision-due:release]")
        );
        assert_eq!(release(sensor, None), Ok(*rules::RELEASES.last().unwrap()));
        assert_eq!(
            release(sensor, Some(Carried::Unreleased)),
            Ok(*rules::RELEASES.last().unwrap())
        );
        assert_eq!(
            release(sensor, Some(Carried::Release(Version(0, 6, 1)))),
            Ok(Version(0, 6, 1))
        );
        // The backlog comes at v0.8.0: a tree of v0.7.0 has none to report as.
        let refusal =
            release(&backlog::SENSOR, Some(Carried::Release(Version(0, 7, 0)))).unwrap_err();
        assert!(refusal.to_string().ends_with("[sense/backlog:release]"));
    }

    #[test]
    fn every_reason_is_one_word_and_names_are_unique() {
        for sensor in ALL {
            assert_eq!(ALL.iter().filter(|s| s.name == sensor.name).count(), 1);
            assert!(sensor.reasons.contains(&RELEASE), "{}", sensor.name);
            assert!(sensor.reasons.contains(&ENVIRONMENT), "{}", sensor.name);
            for reason in sensor.reasons {
                assert!(reason.bytes().all(|b| b.is_ascii_lowercase()), "{reason}");
            }
        }
    }

    #[test]
    fn an_option_is_refused_and_an_extra_argument_by_the_release() {
        assert_eq!(positional(&["a", "b"], 2, true), Some(&["a", "b"][..]));
        assert_eq!(
            positional(&["a", "b", "c"], 2, false),
            Some(&["a", "b"][..])
        );
        assert_eq!(positional(&["a", "b", "c"], 2, true), None);
        assert_eq!(positional(&["a"], 0, false), Some(&[][..]));
        assert_eq!(positional(&["a"], 0, true), None);
        for refuses_extra in [false, true] {
            assert_eq!(positional(&["a", "b", "--x"], 2, refuses_extra), None);
            assert_eq!(positional(&["-x"], 2, refuses_extra), None);
        }
    }
}
