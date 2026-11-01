//! Markdown as the template's `scripts/check-docs.sh` reads it, line by line and byte by byte:
//! the inline links of a file outside fenced code, as its awk pass `scan_markdown` finds them,
//! and the target of one link, as its `link_target` reads it.

use crate::bash::{awk_lines, is_space};

/// Each inline link `[text](target)` of the file outside fenced code, with its line number. A
/// line that opens with space or `>` and then three backquotes or tildes toggles the fence and
/// is not read; the toggle does not ask the closing fence to match the opening one.
pub fn links(text: &[u8]) -> Vec<(usize, &[u8])> {
    let mut fence = false;
    let mut found = Vec::new();
    for (i, line) in awk_lines(text).into_iter().enumerate() {
        if toggles_fence(line) {
            fence = !fence;
            continue;
        }
        if fence {
            continue;
        }
        let mut rest = line;
        while let Some((start, length)) = first_link(rest) {
            found.push((i + 1, &rest[start..start + length]));
            rest = &rest[start + length..];
        }
    }
    found
}

/// Whether the line matches `^[[:space:]>]*(```|~~~)`.
fn toggles_fence(line: &[u8]) -> bool {
    let lead = line
        .iter()
        .take_while(|&&b| is_space(b) || b == b'>')
        .count();
    let rest = &line[lead..];
    rest.starts_with(b"```") || rest.starts_with(b"~~~")
}

/// The leftmost link `\[[^]]*\]\([^)]+\)` of the line, as its start and length.
fn first_link(line: &[u8]) -> Option<(usize, usize)> {
    (0..line.len())
        .filter(|&i| line[i] == b'[')
        .find_map(|start| {
            let close = start + 1 + line[start + 1..].iter().position(|&b| b == b']')?;
            if line.get(close + 1) != Some(&b'(') {
                return None;
            }
            let open = close + 1;
            let end = open + 1 + line[open + 1..].iter().position(|&b| b == b')')?;
            (end > open + 1).then_some((start, end + 1 - start))
        })
}

/// The text of a link, from its opening bracket to the first closing one.
pub fn link_text(link: &[u8]) -> &[u8] {
    let inner = link.strip_prefix(b"[").unwrap_or(link);
    inner
        .iter()
        .position(|&b| b == b']')
        .map_or(inner, |end| &inner[..end])
}

/// The target of a link, trimmed, an optional `"title"` or `'title'` after it removed.
pub fn link_target(link: &[u8]) -> &[u8] {
    let mut target = link;
    let close = link.iter().position(|&b| b == b']');
    if let Some(close) = close.filter(|&c| {
        link.starts_with(b"[") && link[c + 1..].starts_with(b"(") && link.ends_with(b")")
    }) {
        target = &link[close + 2..link.len() - 1];
    }
    let start = target
        .iter()
        .position(|&b| !is_space(b))
        .unwrap_or(target.len());
    let end = target
        .iter()
        .rposition(|&b| !is_space(b))
        .map_or(start, |i| i + 1);
    let target = &target[start..end];
    let path_end = target
        .iter()
        .position(|&b| is_space(b))
        .unwrap_or(target.len());
    let title = &target[path_end..];
    let title = &title[title.iter().take_while(|&&b| is_space(b)).count()..];
    let quoted = |q: u8| {
        title.len() >= 2
            && title[0] == q
            && title[title.len() - 1] == q
            && !title[1..title.len() - 1].contains(&q)
    };
    if path_end > 0 && path_end < target.len() && (quoted(b'"') || quoted(b'\'')) {
        &target[..path_end]
    } else {
        target
    }
}

/// The leftmost link of the line, as its start and length, for a check that cuts a line at its
/// links.
pub fn leftmost_link(line: &[u8]) -> Option<(usize, usize)> {
    first_link(line)
}

/// Whether the line is read for links: outside a fence, and no fence line itself. The lines of
/// the file, each with whether `scan_markdown` reads it.
pub fn scanned_lines(text: &[u8]) -> Vec<(usize, &[u8], bool)> {
    let mut fence = false;
    awk_lines(text)
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            if toggles_fence(line) {
                fence = !fence;
                (i + 1, line, false)
            } else {
                (i + 1, line, !fence)
            }
        })
        .collect()
}

/// A relative link of a Markdown file, as check 3 reads it: its line, its target, the path the
/// bash tests for it, joined to the file's directory as text and not resolved, and its fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub line: usize,
    pub target: Vec<u8>,
    /// The path relative to the root, as `${dir}/${path}` with the root's prefix taken off;
    /// none for an anchor into the file itself.
    pub path: Option<String>,
    pub fragment: Option<Vec<u8>>,
}

/// The relative links of the Markdown file at `file`, a path from the root: every link outside
/// fences but those to `http://`, `https://`, and `mailto:`, and those whose target carries the
/// template's placeholder `NNNN`.
pub fn relative_targets(file: &str, text: &[u8]) -> Vec<Target> {
    let dir = file.rsplit_once('/').map(|(dir, _)| dir);
    links(text)
        .into_iter()
        .filter_map(|(line, link)| {
            let target = link_target(link);
            let skipped = [&b"http://"[..], b"https://", b"mailto:"]
                .iter()
                .any(|p| target.starts_with(p))
                || crate::bash::contains(target, b"NNNN");
            if skipped {
                return None;
            }
            let hash = target.iter().position(|&b| b == b'#');
            let path_part = &target[..hash.unwrap_or(target.len())];
            let fragment = hash.map(|at| target[at + 1..].to_vec());
            let path = (!path_part.is_empty()).then(|| {
                let path_part = String::from_utf8_lossy(path_part);
                match dir {
                    Some(dir) => format!("{dir}/{path_part}"),
                    None => path_part.into_owned(),
                }
            });
            Some(Target {
                line,
                target: target.to_vec(),
                path,
                fragment,
            })
        })
        .collect()
}

/// The anchor of each ATX heading of the file, as the bash's extract_heading_slugs makes it:
/// lower case, every byte but `a-z 0-9 space _ -` dropped, then a hyphen for each space.
pub fn heading_slugs(text: &[u8]) -> Vec<Vec<u8>> {
    awk_lines(text)
        .into_iter()
        .filter_map(|line| {
            let hashes = line.iter().take_while(|&&b| b == b'#').count();
            let spaces = line[hashes..].iter().take_while(|&&b| is_space(b)).count();
            if hashes == 0 || spaces == 0 || hashes > 6 {
                return None;
            }
            let text = strip_closing(&line[hashes + spaces..]);
            Some(
                text.iter()
                    .map(u8::to_ascii_lowercase)
                    .filter(|b| {
                        b.is_ascii_lowercase()
                            || b.is_ascii_digit()
                            || matches!(b, b' ' | b'_' | b'-')
                    })
                    .map(|b| if b == b' ' { b'-' } else { b })
                    .collect(),
            )
        })
        .collect()
}

/// The heading text without a closing run, as `sub(/[[:space:]]+#*[[:space:]]*$/, "")` takes the
/// leftmost such run that reaches the end.
fn strip_closing(text: &[u8]) -> &[u8] {
    let after_spaces = text
        .iter()
        .rposition(|&b| !is_space(b))
        .map_or(0, |i| i + 1);
    let after_hashes = text[..after_spaces]
        .iter()
        .rposition(|&b| b != b'#')
        .map_or(0, |i| i + 1);
    let before = text[..after_hashes]
        .iter()
        .rposition(|&b| !is_space(b))
        .map_or(0, |i| i + 1);
    if after_hashes < after_spaces && before < after_hashes {
        &text[..before]
    } else if after_spaces < text.len() {
        &text[..after_spaces]
    } else {
        text
    }
}

/// Whether a fragment is a line anchor `Lnn` or `Lnn-Lmm`, as the bash's pattern
/// `^L([0-9]+)(-L([0-9]+))?$` matches it.
pub fn is_line_anchor(fragment: &[u8]) -> bool {
    line_numbers(fragment).is_some()
}

/// The digits of a line anchor's start and end, the end the start's when the anchor names one
/// line.
fn line_numbers(fragment: &[u8]) -> Option<(&[u8], &[u8])> {
    let digits = |s: &[u8]| !s.is_empty() && s.iter().all(u8::is_ascii_digit);
    let rest = fragment.strip_prefix(b"L")?;
    let (start, end) = match rest.iter().position(|&b| b == b'-') {
        Some(dash) => (&rest[..dash], rest[dash + 1..].strip_prefix(b"L")?),
        None => (rest, rest),
    };
    (digits(start) && digits(end)).then_some((start, end))
}

/// The value bash's arithmetic gives the digits: octal after a leading zero, decimal otherwise,
/// wrapping at 64 bits as bash does; none for a digit the base lacks, which bash reports as an
/// error that makes the whole test false.
fn bash_number(digits: &[u8]) -> Option<i64> {
    let base: i64 = if digits.len() > 1 && digits[0] == b'0' {
        8
    } else {
        10
    };
    digits.iter().try_fold(0i64, |value, &b| {
        let digit = i64::from(b - b'0');
        (digit < base).then(|| value.wrapping_mul(base).wrapping_add(digit))
    })
}

/// Whether a fragment `Lnn` or `Lnn-Lmm` names lines of the file, a last line without a newline
/// counted, compared as bash's `((…))` compares them; none when the fragment is no line anchor.
pub fn line_anchor_resolves(fragment: &[u8], text: &[u8]) -> Option<bool> {
    let (start, end) = line_numbers(fragment)?;
    let (Some(start), Some(end)) = (bash_number(start), bash_number(end)) else {
        return Some(false);
    };
    let lines = text.iter().filter(|&&b| b == b'\n').count() as i64;
    Some(start >= 1 && end >= start && end <= lines.wrapping_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_found_outside_fences_leftmost_and_in_turn() {
        let text = b"See [a](x.md) and [b](y.md#s).\n```\n[c](z.md)\n```\n> ~~~\n[d](w.md)\n> ~~~\n[[e]](v.md) [f] (u) [g]()\n[x [y](z)\n";
        let found: Vec<(usize, &[u8])> = links(text);
        assert_eq!(
            found,
            [
                (1, &b"[a](x.md)"[..]),
                (1, &b"[b](y.md#s)"[..]),
                (9, &b"[x [y](z)"[..])
            ]
        );
    }

    #[test]
    fn the_target_is_trimmed_and_loses_its_title() {
        assert_eq!(link_target(b"[a]( x.md )"), b"x.md");
        assert_eq!(link_target(b"[a](x.md \"The title\")"), b"x.md");
        assert_eq!(link_target(b"[a](x.md 'The title')"), b"x.md");
        assert_eq!(link_target(b"[a](x.md title)"), b"x.md title");
        assert_eq!(link_text(b"[ADR-x](y.md)"), b"ADR-x");
    }
    #[test]
    fn heading_slugs_are_made_as_the_awk_makes_them() {
        let text = b"# Title ##\n## Bar & Done ##\n### C#\n#### C# \n##### Title#\n####### Seven\n#NoSpace\n## 3. ADR rules\n#\tTab\n## \xc3\x9cber\n";
        let slugs: Vec<Vec<u8>> = heading_slugs(text);
        let expected: [&[u8]; 8] = [
            b"title",
            b"bar--done",
            b"c",
            b"c",
            b"title",
            b"3-adr-rules",
            b"tab",
            b"ber",
        ];
        assert_eq!(slugs, expected.map(<[u8]>::to_vec));
    }

    #[test]
    fn a_line_anchor_names_lines_a_last_one_without_newline_counted() {
        assert_eq!(line_anchor_resolves(b"L2", b"a\nb\nc"), Some(true));
        assert_eq!(line_anchor_resolves(b"L1-L3", b"a\nb\nc"), Some(true));
        assert_eq!(line_anchor_resolves(b"L4", b"a\nb\nc"), Some(false));
        assert_eq!(line_anchor_resolves(b"L3-L2", b"a\nb\nc\n"), Some(false));
        assert_eq!(line_anchor_resolves(b"L0", b"a\n"), Some(false));
        assert_eq!(line_anchor_resolves(b"top", b"a\n"), None);
        let eight = b"1\n2\n3\n4\n5\n6\n7\n8\n";
        assert_eq!(line_anchor_resolves(b"L010", eight), Some(true));
        assert_eq!(line_anchor_resolves(b"L012", eight), Some(false));
        assert_eq!(line_anchor_resolves(b"L09", eight), Some(false));
        assert_eq!(line_anchor_resolves(b"L5-L09", eight), Some(false));
        assert_eq!(
            line_anchor_resolves(b"L99999999999999999999", eight),
            Some(false)
        );
        assert_eq!(
            line_anchor_resolves(b"L18446744073709551617", eight),
            Some(true)
        );
    }

    #[test]
    fn relative_targets_join_the_directory_and_skip_the_web_and_placeholders() {
        let text = b"[a](../x.md#s) [b](#top) [c](https://e.invalid) [d](mailto:m) [e](NNNN-x.md) [f](y/)\n";
        let found = relative_targets("docs/adr/0001-a.md", text);
        let paths: Vec<Option<&str>> = found.iter().map(|t| t.path.as_deref()).collect();
        assert_eq!(paths, [Some("docs/adr/../x.md"), None, Some("docs/adr/y/")]);
        assert_eq!(found[0].fragment.as_deref(), Some(&b"s"[..]));
        assert_eq!(
            relative_targets("README.md", b"[a](x.md)\n")[0]
                .path
                .as_deref(),
            Some("x.md")
        );
    }
}
