//! Part 4 of the template's `scripts/check-traceability.sh` from v0.9.0 on: every
//! `scripts/sensor-<name>.sh` has its self-test `scripts/test-sensor-<name>.sh`, and that self-test
//! carries a marker citing the ADR that decides the checks (NUC-checks-are-bash §4). A tree of an
//! earlier release has no such check, and this rule reports nothing there.

use super::{CATEGORY_IDENTIFIERS, Input, Inputs, Rule};
use crate::finding::Finding;
use crate::traceability;

pub const RULE_ID: &str = "traceability/sensor-tests";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["adr", "absent", "uncited"],
    explain: "Every scripts/sensor-<name>.sh has its self-test scripts/test-sensor-<name>.sh, and \
              that self-test carries a 'Verifies:' marker citing NUC-checks-are-bash, the ADR \
              that decides the checks, whichever else it cites. A sensor reports what no check \
              fails on, so nothing else notices it reporting wrong; its self-test is what does, \
              and the citation ties it to the decision. A tree with a sensor and without that ADR \
              fails once. From template release v0.9.0 on; earlier trees have no such check.",
    reads: &[Input::Sensors, Input::Markers, Input::Release],
    check: |inputs| check(inputs).into(),
};

/// The ADR that decides the checks, which the self-test of each sensor cites.
pub const CHECKS_ADR: &str = "NUC-checks-are-bash";

/// What the shell reads of the sensors: each `scripts/sensor-<name>.sh` git lists as tracked or
/// new and not ignored that is a file, in git's order, with whether its self-test is a file; and
/// whether the ADR that decides the checks is a file of `docs/adr/`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sensors {
    pub sensors: Vec<(String, bool)>,
    pub checks_adr: bool,
}

/// The name of a sensor, as `^scripts/sensor-([^/]+)\.sh$` captures it from its path.
pub fn sensor_name(path: &str) -> Option<&str> {
    path.strip_prefix("scripts/sensor-")?
        .strip_suffix(".sh")
        .filter(|name| !name.is_empty() && !name.contains('/'))
}

/// The self-test of a sensor's path.
pub fn self_test(name: &str) -> String {
    format!("scripts/test-sensor-{name}.sh")
}

fn finding(reason: &'static str, file: String, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file,
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    if !inputs.carries_at_least(CATEGORY_IDENTIFIERS) || inputs.sensors.sensors.is_empty() {
        return Vec::new();
    }
    if !inputs.sensors.checks_adr {
        return vec![finding(
            "adr",
            format!("docs/adr/{CHECKS_ADR}.md"),
            "not found — the self-test of each sensor cites it".to_owned(),
        )];
    }
    let citations = traceability::citations(&inputs.markers, true);
    let mut found = Vec::new();
    for (path, tested) in &inputs.sensors.sensors {
        let Some(name) = sensor_name(path) else {
            continue;
        };
        let test = self_test(name);
        if !tested {
            found.push(finding(
                "absent",
                path.clone(),
                format!(
                    "has no self-test {test} — each sensor runs after one that cites {CHECKS_ADR}"
                ),
            ));
        } else if !citations
            .iter()
            .any(|c| c.file == test && c.id == CHECKS_ADR)
        {
            found.push(finding(
                "uncited",
                test,
                format!(
                    "does not cite {CHECKS_ADR} — the self-test of a sensor cites the ADR that \
                     decides the checks"
                ),
            ));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::testing::{cat_id, mark};

    fn inputs(sensors: &[(&str, bool)], checks_adr: bool, marks: &[(&str, &str)]) -> Inputs {
        Inputs {
            sensors: Sensors {
                sensors: sensors.iter().map(|&(p, t)| (p.to_owned(), t)).collect(),
                checks_adr,
            },
            markers: marks
                .iter()
                .map(|&(f, ids)| (f.to_owned(), mark(ids).into_bytes()))
                .collect(),
            ..Inputs::default()
        }
    }

    fn reasons(inputs: &Inputs) -> Vec<(&'static str, String)> {
        check(inputs)
            .into_iter()
            .map(|f| (f.reason, f.file))
            .collect()
    }

    #[test]
    fn each_sensor_needs_a_self_test_that_cites_the_adr_on_the_checks() {
        let other = cat_id("SEC", "host-socket-closed");
        let both = format!("{other} {CHECKS_ADR}");
        let found = inputs(
            &[
                ("scripts/sensor-a.sh", false),
                ("scripts/sensor-b.sh", true),
                ("scripts/sensor-c.sh", true),
            ],
            true,
            &[
                ("scripts/test-sensor-b.sh", other.as_str()),
                ("scripts/test-sensor-c.sh", both.as_str()),
            ],
        );
        assert_eq!(
            reasons(&found),
            [
                ("absent", "scripts/sensor-a.sh".to_owned()),
                ("uncited", "scripts/test-sensor-b.sh".to_owned()),
            ]
        );
        let messages: Vec<String> = check(&found).into_iter().map(|f| f.message).collect();
        assert_eq!(
            messages,
            [
                format!(
                    "has no self-test scripts/test-sensor-a.sh — each sensor runs after one that \
                     cites {CHECKS_ADR}"
                ),
                format!(
                    "does not cite {CHECKS_ADR} — the self-test of a sensor cites the ADR that \
                     decides the checks"
                ),
            ]
        );
    }

    #[test]
    fn sensors_without_the_adr_on_the_checks_fail_once_and_no_sensor_passes() {
        let without = inputs(
            &[
                ("scripts/sensor-a.sh", false),
                ("scripts/sensor-b.sh", false),
            ],
            false,
            &[],
        );
        assert_eq!(
            reasons(&without),
            [("adr", format!("docs/adr/{CHECKS_ADR}.md"))]
        );
        assert!(reasons(&inputs(&[], false, &[])).is_empty());
    }

    #[test]
    fn before_v0_9_0_there_is_no_such_check() {
        let mut earlier = inputs(&[("scripts/sensor-a.sh", false)], false, &[]);
        earlier.release = Some(Version(0, 8, 1));
        assert!(reasons(&earlier).is_empty());
    }

    #[test]
    fn a_sensor_is_named_as_the_bash_captures_it() {
        assert_eq!(sensor_name("scripts/sensor-x.sh"), Some("x"));
        assert_eq!(sensor_name("scripts/sensor-a.b.sh"), Some("a.b"));
        assert_eq!(sensor_name("scripts/sensor-.sh"), None);
        assert_eq!(sensor_name("scripts/lib/sensor-x.sh"), None);
        assert_eq!(sensor_name("scripts/sensor-a/b.sh"), None);
        assert_eq!(sensor_name("scripts/test-sensor-x.sh"), None);
    }
}
