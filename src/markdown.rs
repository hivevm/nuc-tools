//! Markdown as the template's `scripts/check-docs.sh` reads it, line by line and byte by byte:
//! the inline links of a file outside fenced code, as its awk pass `scan_markdown` finds them,
//! and the target of one link, as its `link_target` reads it.

use crate::adr::{awk_lines, is_space};

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
}
