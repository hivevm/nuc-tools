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

/// The fields `IFS=$'\t' read -r a rest` splits a line into: the first run of bytes between tabs,
/// and what follows it without the tabs that lead or end it, as [`read_tab_fields`] reads them.
pub fn read_tab_pair(line: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let line: Vec<u8> = line.iter().copied().filter(|&b| b != 0).collect();
    let tab = |b: &u8| *b == b'\t';
    let start = line.iter().position(|b| !tab(b)).unwrap_or(line.len());
    let end = line[start..]
        .iter()
        .position(tab)
        .map_or(line.len(), |i| start + i);
    let rest = &line[end..];
    let from = rest.iter().position(|b| !tab(b)).unwrap_or(rest.len());
    let to = rest.iter().rposition(|b| !tab(b)).map_or(from, |i| i + 1);
    (line[start..end].to_vec(), rest[from..to].to_vec())
}

/// The fields awk splits a line into with its default field separator: the runs of bytes between
/// blanks, spaces and tabs, those that lead or end the line left out.
pub fn awk_fields(line: &[u8]) -> Vec<&[u8]> {
    line.split(|&b| b == b' ' || b == b'\t' || b == b'\n')
        .filter(|field| !field.is_empty())
        .collect()
}

/// The bytes as a command substitution `$(...)` gives them: every newline that ends them dropped.
pub fn substituted(text: &[u8]) -> &[u8] {
    let end = text.iter().rposition(|&b| b != b'\n').map_or(0, |i| i + 1);
    &text[..end]
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

/// The largest whole number awk's double holds exactly, 2^53: past it awk reads, adds, and prints
/// another number than the digits say.
pub const AWK_EXACT: u64 = 1 << 53;

/// A number past [`AWK_EXACT`], which awk reads inexactly and nuc does not read: the reading that
/// meets one is not decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow;

/// The number awk reads at the start of the bytes: their leading digits, 0 where none stand; past
/// [`AWK_EXACT`] an overflow. A sign, which git never writes, is no digit here.
pub fn awk_number(bytes: &[u8]) -> Result<u64, Overflow> {
    bytes
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .try_fold(0u64, |n, &b| {
            n.checked_mul(10)
                .and_then(|n| n.checked_add(u64::from(b - b'0')))
                .filter(|&n| n <= AWK_EXACT)
                .ok_or(Overflow)
        })
}

/// The sum awk computes of the numbers, where it stays within [`AWK_EXACT`].
pub fn awk_sum(numbers: &[u64]) -> Result<u64, Overflow> {
    numbers
        .iter()
        .try_fold(0u64, |sum, &n| sum.checked_add(n))
        .filter(|&sum| sum <= AWK_EXACT)
        .ok_or(Overflow)
}

/// The first `+<digits>` of a hunk's head, anywhere on the line, as awk's
/// `match($0, /\+[0-9]+/)` and `+ 0` read it; 0 where the line has none.
pub fn hunk_first_added(line: &[u8]) -> Result<u64, Overflow> {
    for at in (0..line.len()).filter(|&at| line[at] == b'+') {
        let rest = &line[at + 1..];
        if rest.first().is_some_and(u8::is_ascii_digit) {
            return awk_number(rest);
        }
    }
    Ok(0)
}

/// The new side of a hunk's head as `sed -nE 's/^@@ [^+]*\+([0-9,]+) .*/\1/p'` gives it and awk
/// splits it at the comma: the first line and the count, a count missing or empty read as 1. None
/// where the line is no such head.
pub fn hunk_new_side(line: &[u8]) -> Result<Option<(u64, u64)>, Overflow> {
    let Some(rest) = line.strip_prefix(b"@@ ") else {
        return Ok(None);
    };
    // `[^+]*` runs to the first `+`; `[0-9,]+` and a space must follow it.
    let Some(plus) = rest.iter().position(|&b| b == b'+') else {
        return Ok(None);
    };
    let after = &rest[plus + 1..];
    let len = after
        .iter()
        .take_while(|&&b| b.is_ascii_digit() || b == b',')
        .count();
    if len == 0 || after.get(len) != Some(&b' ') {
        return Ok(None);
    }
    let mut parts = after[..len].split(|&b| b == b',');
    let start = awk_number(parts.next().unwrap_or_default())?;
    let count = match parts.next().filter(|count| !count.is_empty()) {
        Some(count) => awk_number(count)?,
        None => 1,
    };
    Ok(Some((start, count)))
}

/// The new side of a hunk's head as awk reads its third field on a line that starts with `@@ `:
/// the field's first byte dropped, the first line and the count, a count missing read as 1 and an
/// empty one as 0. None where the line is no such head.
pub fn hunk_third_field(line: &[u8]) -> Result<Option<(u64, u64)>, Overflow> {
    if !line.starts_with(b"@@ ") {
        return Ok(None);
    }
    let Some(new) = line
        .split(|&b| b == b' ' || b == b'\t')
        .filter(|f| !f.is_empty())
        .nth(2)
    else {
        return Ok(None);
    };
    let mut parts = new.split(|&b| b == b',');
    let start = awk_number(
        parts
            .next()
            .map_or(&[][..], |p| p.get(1..).unwrap_or_default()),
    )?;
    let count = match parts.next() {
        Some(count) => awk_number(count)?,
        None => 1,
    };
    Ok(Some((start, count)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awk_reads_the_leading_digits_and_nuc_refuses_a_number_awk_reads_inexactly() {
        assert_eq!(awk_number(b"12ab"), Ok(12));
        assert_eq!(awk_number(b"x"), Ok(0));
        assert_eq!(awk_number(b""), Ok(0));
        assert_eq!(awk_number(b"9007199254740992"), Ok(AWK_EXACT));
        assert_eq!(awk_number(b"9007199254740993"), Err(Overflow));
        assert_eq!(awk_number(b"99999999999999999999"), Err(Overflow));
        assert_eq!(awk_sum(&[AWK_EXACT - 1, 1]), Ok(AWK_EXACT));
        assert_eq!(awk_sum(&[AWK_EXACT, 1]), Err(Overflow));
    }

    #[test]
    fn each_awk_pass_reads_a_hunk_head_with_its_quirk() {
        // tests-kept: the first `+<digits>` anywhere on the line.
        assert_eq!(hunk_first_added(b"@@ -1 +3,2 @@ f(+9)"), Ok(3));
        assert_eq!(hunk_first_added(b"@@ -1+x +7 @@"), Ok(7));
        assert_eq!(hunk_first_added(b"no hunk"), Ok(0));
        assert_eq!(
            hunk_first_added(b"@@ -1 +99999999999999999999 @@"),
            Err(Overflow)
        );
        // binding-adrs: the sed pattern, an empty or missing count as 1.
        assert_eq!(hunk_new_side(b"@@ -1 +3,2 @@ x"), Ok(Some((3, 2))));
        assert_eq!(hunk_new_side(b"@@ -1 +3 @@"), Ok(Some((3, 1))));
        assert_eq!(hunk_new_side(b"@@ -1 +3, @@"), Ok(Some((3, 1))));
        assert_eq!(hunk_new_side(b"@@ -1 +3,2"), Ok(None));
        // sed's first `+` decides, and it needs a digit or a comma after it.
        assert_eq!(hunk_new_side(b"@@ -1 +x +3 @@"), Ok(None));
        assert_eq!(hunk_new_side(b"@@ -1 + @@"), Ok(None));
        assert_eq!(
            hunk_new_side(b"@@ -1 +99999999999999999999 @@"),
            Err(Overflow)
        );
        // plain-rules: the third field after `@@ `, a missing count as 1, an empty one as 0.
        assert_eq!(hunk_third_field(b"@@ -1 +3,2 @@"), Ok(Some((3, 2))));
        assert_eq!(hunk_third_field(b"@@ -1 +3 @@"), Ok(Some((3, 1))));
        assert_eq!(hunk_third_field(b"@@ -1 +3, @@"), Ok(Some((3, 0))));
        assert_eq!(hunk_third_field(b"@@ -1"), Ok(None));
        assert_eq!(
            hunk_third_field(b"@@ -1 +99999999999999999999 @@"),
            Err(Overflow)
        );
    }

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

    #[test]
    fn a_tab_pair_keeps_the_spaces_and_trims_the_tabs_of_the_rest() {
        assert_eq!(
            read_tab_pair(b"a b\t\tc d\t"),
            (b"a b".to_vec(), b"c d".to_vec())
        );
        assert_eq!(read_tab_pair(b"\tk"), (b"k".to_vec(), Vec::new()));
        assert_eq!(read_tab_pair(b"x\ty\tz"), (b"x".to_vec(), b"y\tz".to_vec()));
    }

    #[test]
    fn awk_fields_split_at_blanks_and_a_substitution_drops_the_newlines_that_end() {
        assert_eq!(awk_fields(b" a\t b "), [&b"a"[..], b"b"]);
        assert_eq!(substituted(b"a\n\nb\n\n"), b"a\n\nb");
        assert_eq!(substituted(b"\n"), b"");
    }
}
