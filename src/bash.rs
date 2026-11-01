//! The bash's tools as the checks read bytes with them: lines as awk, `while IFS= read -r`, and
//! `sed` split a file, the space class of POSIX that `[[:space:]]` matches, and the word `read`
//! splits off with the default IFS. Each reads some bytes otherwise than Rust's own readings, and
//! a port that writes its own copy diverges where they differ; the structural test of
//! `tests/functional_core.rs` fails on a function of one of these names defined elsewhere.

/// The lines of the text as awk reads them: split at newlines, an unterminated last line kept.
pub fn awk_lines(text: &[u8]) -> Vec<&[u8]> {
    let mut lines: Vec<&[u8]> = text.split(|&b| b == b'\n').collect();
    if text.ends_with(b"\n") || text.is_empty() {
        lines.pop();
    }
    lines
}

/// The lines of the text as `while IFS= read -r` reads them: an unterminated last line dropped.
pub fn read_lines(text: &[u8]) -> Vec<&[u8]> {
    let mut lines: Vec<&[u8]> = text.split(|&b| b == b'\n').collect();
    lines.pop();
    lines
}

/// The lines `sed` reads: every line, the last one without a newline included.
pub fn sed_lines(text: &[u8]) -> impl Iterator<Item = &[u8]> {
    let text = text.strip_suffix(b"\n").unwrap_or(text);
    text.split(|&b| b == b'\n')
        .filter(move |_| !text.is_empty())
}

/// Whether the byte is in the space class of POSIX, which the bash matches with `[[:space:]]`.
pub fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

/// Whether the character is in the space class of POSIX, as [`is_space`] for a byte.
pub fn is_space_char(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
}

/// The bytes without leading and trailing POSIX space.
pub fn trim(text: &[u8]) -> &[u8] {
    let start = text
        .iter()
        .position(|&b| !is_space(b))
        .unwrap_or(text.len());
    let end = text
        .iter()
        .rposition(|&b| !is_space(b))
        .map_or(start, |i| i + 1);
    &text[start..end]
}

/// The first word of the bytes as `read -r word _` splits them with the default IFS: space, tab,
/// and newline, and nothing else.
pub fn first_word(text: &[u8]) -> &[u8] {
    let ifs = |b: &u8| matches!(b, b' ' | b'\t' | b'\n');
    let start = text.iter().position(|b| !ifs(b)).unwrap_or(text.len());
    let end = text[start..]
        .iter()
        .position(ifs)
        .map_or(text.len(), |i| start + i);
    &text[start..end]
}

/// The position of the first occurrence of the needle in the bytes; an empty needle is at 0.
pub fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Whether the needle occurs in the bytes.
pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    find(haystack, needle).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_drops_an_unterminated_last_line_and_awk_keeps_it() {
        assert_eq!(read_lines(b"a\nb"), [&b"a"[..]]);
        assert_eq!(awk_lines(b"a\nb"), [&b"a"[..], &b"b"[..]]);
        assert_eq!(awk_lines(b"a\n"), [&b"a"[..]]);
    }

    #[test]
    fn the_first_word_splits_at_space_tab_and_newline_only() {
        assert_eq!(first_word(b" accepted\r"), b"accepted\r");
        assert_eq!(
            first_word("\u{a0}accepted".as_bytes()),
            "\u{a0}accepted".as_bytes()
        );
        assert_eq!(first_word(b"\x0Baccepted"), b"\x0Baccepted");
        assert_eq!(first_word(b"\taccepted by X"), b"accepted");
    }

    #[test]
    fn awk_keeps_a_carriage_return_that_rust_s_lines_drops() {
        assert_eq!(awk_lines(b"a\r\nb"), [&b"a\r"[..], &b"b"[..]]);
        assert_eq!("a\r\nb".lines().collect::<Vec<_>>(), ["a", "b"]);
    }

    #[test]
    fn sed_reads_an_unterminated_last_line_that_read_drops() {
        assert_eq!(
            sed_lines(b"a\nb").collect::<Vec<_>>(),
            [&b"a"[..], &b"b"[..]]
        );
        assert_eq!(read_lines(b"a\nb"), [&b"a"[..]]);
        assert_eq!(sed_lines(b"").count(), 0);
    }

    #[test]
    fn posix_space_holds_the_vertical_tab_that_rust_s_ascii_whitespace_lacks() {
        assert!(is_space(0x0B));
        assert!(!0x0Bu8.is_ascii_whitespace());
        assert!(!is_space(0xA0));
    }

    #[test]
    fn posix_space_of_a_character_lacks_the_no_break_space_rust_counts() {
        assert!(is_space_char('\x0B'));
        assert!(!is_space_char('\u{a0}'));
        assert!('\u{a0}'.is_whitespace());
    }

    #[test]
    fn trim_takes_the_vertical_tab_and_keeps_a_no_break_space() {
        assert_eq!(trim(b"\x0B a \r"), b"a");
        assert_eq!(b"\x0B a".trim_ascii(), b"\x0B a");
        assert_eq!(trim("\u{a0}a".as_bytes()), "\u{a0}a".as_bytes());
    }
}
