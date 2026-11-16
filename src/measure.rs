//! The counting of `nuc measure`: the tokens a harness's sessions on a branch read and wrote, as
//! the Measures line of a hand-off counts them (API-measure-reads-harness-transcripts). Pure: the
//! shell reads the transcripts and hands their bytes over, an adapter per harness turns them into
//! requests, and the counting knows no harness.

pub mod claude_code;

use std::collections::BTreeMap;

/// The tokens of one request as the harness reports them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    /// The input tokens not read from the cache.
    pub input: u64,
    /// The input tokens read from the cache.
    pub cache_read: u64,
    /// The input tokens written to the cache.
    pub cache_written: u64,
    /// The output tokens.
    pub output: u64,
}

impl Usage {
    /// What the request read and wrote: the four counts summed.
    #[must_use]
    pub fn total(self) -> u64 {
        self.input + self.cache_read + self.cache_written + self.output
    }

    /// Each count the larger of the two: the lines of one request report it as it streams, so its
    /// last line, and the largest count, is the request's.
    #[must_use]
    fn largest(self, other: Self) -> Self {
        Self {
            input: self.input.max(other.input),
            cache_read: self.cache_read.max(other.cache_read),
            cache_written: self.cache_written.max(other.cache_written),
            output: self.output.max(other.output),
        }
    }
}

/// One line of a transcript that reports a request of the model: the request's identifier, the
/// branch the session was on, and the usage the line reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub id: String,
    pub branch: String,
    pub usage: Usage,
}

/// The harnesses whose transcripts `nuc measure` reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Harness {
    ClaudeCode,
}

impl Harness {
    /// The harness an option names, none for a name `nuc` knows no adapter for.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        (name == "claude-code").then_some(Self::ClaudeCode)
    }

    /// The requests the lines of one transcript report, or why it cannot be read: a line that
    /// does not parse or a request without its usage, at its file and line. `name` names the
    /// transcript in a refusal.
    ///
    /// # Errors
    ///
    /// The refusal, which names the file and the line and nothing of the transcript's text.
    pub fn requests(self, name: &str, bytes: &[u8]) -> Result<Vec<Request>, String> {
        match self {
            Self::ClaudeCode => claude_code::requests(name, bytes),
        }
    }
}

/// The tokens the requests on `branch` read and wrote, each request counted once with the largest
/// usage its lines report; none where no request was made on it.
#[must_use]
pub fn tokens(requests: &[Request], branch: &str) -> Option<u64> {
    let mut by_id: BTreeMap<&str, Usage> = BTreeMap::new();
    for request in requests.iter().filter(|request| request.branch == branch) {
        let usage = by_id.entry(&request.id).or_default();
        *usage = usage.largest(request.usage);
    }
    (!by_id.is_empty()).then(|| by_id.values().map(|usage| usage.total()).sum())
}

/// The part of the Measures line `nuc measure` prints, ready to paste.
#[must_use]
pub fn line(tokens: u64) -> String {
    format!("tokens {tokens}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str, branch: &str, output: u64) -> Request {
        Request {
            id: id.to_owned(),
            branch: branch.to_owned(),
            usage: Usage {
                input: 1,
                cache_read: 10,
                cache_written: 100,
                output,
            },
        }
    }

    #[test]
    fn a_request_over_several_lines_counts_once_at_its_largest_usage() {
        let lines = [
            request("r", "b", 8),
            request("r", "b", 935),
            request("r", "b", 900),
        ];
        assert_eq!(tokens(&lines, "b"), Some(1 + 10 + 100 + 935));
    }

    #[test]
    fn only_the_requests_of_the_branch_count() {
        let lines = [request("r1", "a", 1), request("r2", "b", 2)];
        assert_eq!(tokens(&lines, "b"), Some(111 + 2));
        assert_eq!(tokens(&lines, "c"), None);
    }

    #[test]
    fn the_line_is_the_measures_part_to_paste() {
        assert_eq!(line(1234), "tokens 1234");
    }

    #[test]
    fn only_claude_code_is_a_harness() {
        assert_eq!(Harness::parse("claude-code"), Some(Harness::ClaudeCode));
        assert_eq!(Harness::parse("Claude Code"), None);
    }
}
