//! The bash's tools as the checks read bytes with them: lines as awk, `while IFS= read -r`, and
//! `sed` split a file, the space class of POSIX that `[[:space:]]` matches, and the word `read`
//! splits off with the default IFS. Each reads some bytes otherwise than Rust's own readings, and
//! a port that writes its own copy diverges where they differ; the structural test of
//! `tests/functional_core.rs` fails on a function of one of these names defined elsewhere. The
//! module also holds `find` and `contains`, the byte search the readers and the rules use, which
//! read no byte otherwise and are no reader the test holds.

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

/// The fields `read -r a b rest` splits a line into with the default IFS: the first two words, and
/// what follows them without the IFS whitespace that leads or ends it; an absent field is empty.
/// `read` drops each NUL byte of the line.
pub fn read_fields(line: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let ifs = |b: &u8| matches!(b, b' ' | b'\t' | b'\n');
    let line: Vec<u8> = line.iter().copied().filter(|&b| b != 0).collect();
    let mut rest: &[u8] = &line;
    let mut word = || {
        let start = rest.iter().position(|b| !ifs(b)).unwrap_or(rest.len());
        let end = rest[start..]
            .iter()
            .position(ifs)
            .map_or(rest.len(), |i| start + i);
        let word = rest[start..end].to_vec();
        rest = &rest[end..];
        word
    };
    let (a, b) = (word(), word());
    let start = rest.iter().position(|b| !ifs(b)).unwrap_or(rest.len());
    let end = rest.iter().rposition(|b| !ifs(b)).map_or(start, |i| i + 1);
    (a, b, rest[start..end].to_vec())
}

/// The fields `read -r a rest` splits a line into with the default IFS: the first word, and what
/// follows it without the IFS whitespace that leads or ends it; an absent field is empty. `read`
/// drops each NUL byte of the line.
pub fn read_pair(line: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let ifs = |b: &u8| matches!(b, b' ' | b'\t' | b'\n');
    let line: Vec<u8> = line.iter().copied().filter(|&b| b != 0).collect();
    let start = line.iter().position(|b| !ifs(b)).unwrap_or(line.len());
    let end = line[start..]
        .iter()
        .position(ifs)
        .map_or(line.len(), |i| start + i);
    let rest = &line[end..];
    let from = rest.iter().position(|b| !ifs(b)).unwrap_or(rest.len());
    let to = rest.iter().rposition(|b| !ifs(b)).map_or(from, |i| i + 1);
    (line[start..end].to_vec(), rest[from..to].to_vec())
}

/// The fields `IFS=$'\t' read -r a b rest` splits a line into: the first two runs of bytes between
/// tabs, and what follows them without the tabs that lead or end it; tabs at either end and runs of
/// them count as one, since a tab is IFS whitespace. An absent field is empty. `read` drops each
/// NUL byte of the line.
pub fn read_tab_fields(line: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let line: Vec<u8> = line.iter().copied().filter(|&b| b != 0).collect();
    let tab = |b: &u8| *b == b'\t';
    let mut rest: &[u8] = &line;
    let mut word = || {
        let start = rest.iter().position(|b| !tab(b)).unwrap_or(rest.len());
        let end = rest[start..]
            .iter()
            .position(tab)
            .map_or(rest.len(), |i| start + i);
        let word = rest[start..end].to_vec();
        rest = &rest[end..];
        word
    };
    let (a, b) = (word(), word());
    let start = rest.iter().position(|b| !tab(b)).unwrap_or(rest.len());
    let end = rest.iter().rposition(|b| !tab(b)).map_or(start, |i| i + 1);
    (a, b, rest[start..end].to_vec())
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
    fn read_pair_splits_off_the_first_word_and_trims_the_rest_by_ifs_alone() {
        assert_eq!(
            read_pair(b"      3 a b \t"),
            (b"3".to_vec(), b"a b".to_vec())
        );
        assert_eq!(read_pair(b"1 x\r"), (b"1".to_vec(), b"x\r".to_vec()));
        assert_eq!(read_pair(b"  "), (Vec::new(), Vec::new()));
    }

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
    fn read_splits_two_words_and_a_rest_trimmed_of_its_ifs_whitespace() {
        let fields = |a: &[u8], b: &[u8], c: &[u8]| (a.to_vec(), b.to_vec(), c.to_vec());
        assert_eq!(read_fields(b" a\tb  c d \t"), fields(b"a", b"b", b"c d"));
        assert_eq!(read_fields(b"a"), fields(b"a", b"", b""));
        assert_eq!(read_fields(b"a b\r"), fields(b"a", b"b\r", b""));
        assert_eq!(read_fields(b"a b \r"), fields(b"a", b"b", b"\r"));
        assert_eq!(
            read_fields(b"a\0b c\0d  e f "),
            fields(b"ab", b"cd", b"e f")
        );
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
