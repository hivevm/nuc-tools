//! One part of a path pattern matched against one name, as bash's pathname expansion matches it
//! in the C locale with its default options: `*` any bytes, `?` one byte, `[…]` one byte of a
//! set, `\` quoting the next byte, and a leading `.` of a name matched only by a `.` written out.

/// Whether the part holds a character that makes it a pattern, rather than a name.
pub fn has_magic(part: &str) -> bool {
    let bytes = part.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'*' | b'?' | b'[' => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

/// The name a part without magic names: its quoting backslashes taken out.
pub fn unescape(part: &str) -> String {
    let mut name = String::new();
    let mut chars = part.chars();
    while let Some(c) = chars.next() {
        name.push(if c == '\\' {
            chars.next().unwrap_or('\\')
        } else {
            c
        });
    }
    name
}

/// Whether the part matches the name; none for a bracket with a character class, an
/// equivalence class, or a collating symbol, which `nuc` does not read.
pub fn matches(part: &str, name: &str) -> Option<bool> {
    let (p, n) = (part.as_bytes(), name.as_bytes());
    if n.first() == Some(&b'.') && !p.starts_with(b".") && !p.starts_with(b"\\.") {
        return Some(false);
    }
    match_from(p, n)
}

fn match_from(p: &[u8], n: &[u8]) -> Option<bool> {
    let Some((&first, rest)) = p.split_first() else {
        return Some(n.is_empty());
    };
    match first {
        b'*' => {
            for skip in 0..=n.len() {
                if match_from(rest, &n[skip..])? {
                    return Some(true);
                }
            }
            Some(false)
        }
        b'?' => match n.split_first() {
            Some((_, n_rest)) => match_from(rest, n_rest),
            None => Some(false),
        },
        b'[' => match bracket(rest)? {
            Some((set, after)) => match n.split_first() {
                Some((&b, n_rest)) if set.contains(b) => match_from(after, n_rest),
                _ => Some(false),
            },
            // An unclosed bracket stands for itself.
            None => literal(b'[', rest, n),
        },
        b'\\' => match rest.split_first() {
            Some((&quoted, after)) => literal(quoted, after, n),
            None => literal(b'\\', rest, n),
        },
        b => literal(b, rest, n),
    }
}

fn literal(b: u8, rest: &[u8], n: &[u8]) -> Option<bool> {
    match n.split_first() {
        Some((&c, n_rest)) if c == b => match_from(rest, n_rest),
        _ => Some(false),
    }
}

/// A byte set of a bracket.
struct Set {
    negated: bool,
    ranges: Vec<(u8, u8)>,
}

impl Set {
    fn contains(&self, b: u8) -> bool {
        self.ranges.iter().any(|&(lo, hi)| lo <= b && b <= hi) != self.negated
    }
}

/// The set of the bracket whose text follows its `[`, and what follows its `]`; none inside when
/// it is not closed, and none outside for a class `nuc` does not read.
fn bracket(p: &[u8]) -> Option<Option<(Set, &[u8])>> {
    let mut i = 0;
    let negated = matches!(p.first(), Some(b'!' | b'^'));
    if negated {
        i += 1;
    }
    let mut ranges = Vec::new();
    let start = i;
    loop {
        let Some(&b) = p.get(i) else {
            return Some(None);
        };
        if b == b']' && i > start {
            return Some(Some((Set { negated, ranges }, &p[i + 1..])));
        }
        if b == b'[' && matches!(p.get(i + 1), Some(b':' | b'=' | b'.')) {
            return None;
        }
        let (lo, next) = if b == b'\\' && i + 1 < p.len() {
            (p[i + 1], i + 2)
        } else {
            (b, i + 1)
        };
        if p.get(next) == Some(&b'-') && p.get(next + 1).is_some_and(|&c| c != b']') {
            let hi = p[next + 1];
            ranges.push((lo, hi));
            i = next + 2;
        } else {
            ranges.push((lo, lo));
            i = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stars_questions_and_brackets_match_as_in_bash() {
        assert_eq!(matches("*.sh", "check.sh"), Some(true));
        assert_eq!(matches("*.sh", "check.shx"), Some(false));
        assert_eq!(matches("*", ".hidden"), Some(false));
        assert_eq!(matches(".*", ".hidden"), Some(true));
        assert_eq!(matches("?", "."), Some(false));
        assert_eq!(matches("a?c", "abc"), Some(true));
        assert_eq!(matches("[a-c]x", "bx"), Some(true));
        assert_eq!(matches("[!a-c]x", "bx"), Some(false));
        assert_eq!(matches("[]a]", "]"), Some(true));
        assert_eq!(matches("[a-]", "-"), Some(true));
        assert_eq!(matches("[ab", "[ab"), Some(true));
        assert_eq!(matches("\\*", "*"), Some(true));
        assert_eq!(matches("\\*", "x"), Some(false));
        assert_eq!(matches("[[:alpha:]]", "a"), None);
    }

    #[test]
    fn a_part_without_magic_is_a_name() {
        assert!(has_magic("a*"));
        assert!(!has_magic("a\\*"));
        assert_eq!(unescape("a\\*b"), "a*b");
    }
}
