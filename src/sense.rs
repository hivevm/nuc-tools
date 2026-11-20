//! The sensors `nuc sense` ports from the template (API-git-only-sensors-under-sense): each a pure
//! function of the values the shell gathered, which returns its report, the bytes the bash sensor
//! of the tree's release writes on the standard output, or a refusal. A sensor reads no file and
//! runs no process; the shell reads, prints, and exits (API-command-line-with-sense).

pub mod action_refs;
pub mod backlog;
pub mod binding_adrs;
pub mod change;
pub mod plain_rules;
pub mod plan_frontier;
pub mod review_binding;
pub mod revision_due;
pub mod tests_kept;

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
/// the network, are none of them (API-git-only-sensors-under-sense).
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
///   it, and ignored before, as the bash before ignores it (API-git-only-sensors-under-sense).
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

#[cfg(test)]
mod tests {
    use super::*;

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
