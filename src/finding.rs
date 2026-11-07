//! A finding: one violation a rule reports, named by its rule ID and its `file:line` (G-6).

use std::fmt;

/// One violation of one rule, at a file and, where the rule can name one, a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

/// The text form, one finding per line: `<file>[:<line>]: <rule ID>: <message>`. The parity
/// harness reads it back by this shape.
impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "{}:{line}: {}: {}", self.file, self.rule, self.message),
            None => write!(f, "{}: {}: {}", self.file, self.rule, self.message),
        }
    }
}
