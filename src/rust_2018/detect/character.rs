//! Detects a char literal, like `'A'` or `\u{03aB}`.

/// Detects a char literal, like `'A'` or `\u{03aB}`.
/// 
/// @TODO `b` prefix, eg `b'A'`
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `pos` The character position in `orig` to look at
/// 
/// ### Returns
/// If `pos` begins a valid looking char literal, `detect_character()`
/// returns the character position after the closing single quote.  
/// Otherwise, `detect_character()` just returns the `pos` argument.
pub fn detect_character(orig: &str, pos: usize) -> usize {
    // Avoid panicking, if there would not be enough room for a char.
    let len = orig.len();
    if len < pos + 3 { return pos } // pos + ' + A + '
    // If the current char is not a single-quote, then it does not begin a char.
    let c0 = get_aot(orig, pos);
    if c0 != "'" { return pos }
    // Get the next char, even if it’s not ascii.
    let mut c1_end = pos + 2;
    while !orig.is_char_boundary(c1_end) { c1_end += 1 }
    // Avoid panicking, if there would not be enough room for a char.
    if len < c1_end + 1 { return pos }
    let c1 = &orig[pos+1..c1_end];
    // If the next char is not a backslash:
    if c1 != "\\" {
        return
            // If `c1` is a single quote:
            if c1 == "'"
                // We have found the string "''", which is not a valid char.
                { pos }
            // Otherwise, if the char directly after `c1` is not a single quote:
            else if get_aot(orig, c1_end) != "'"
                // We have probably found a label, like "'static".
                { pos }
            // Otherwise, this is a valid char literal, like "'A'" or "'±'".
            else { c1_end + 1 }
    }

    // Now we know `c1` is a backslash, if the char after it is...
    match get_aot(orig, pos+2) {
        // ...one of Rust’s simple backslashable chars:
        "n" | "r" | "t" | "\\" | "0" | "\"" | "'" =>
            // Advance four places if the char after that is a single-quote.
            pos +
                if len >= pos + 4
                && get_aot(orig, pos+3) == "'"
                { 4 } else { 0 },
        // ...lowercase x, signifying a 7-bit char code:
        "x" =>
            // Advance 6 places if the chars after that are 0-7 and 0-9A-Fa-f.
            pos +
                if len >= pos + 6
                && get_aot(orig, pos+3).chars().all(|c| c >= '0' && c <= '7')
                && get_aot(orig, pos+4).chars().all(|c| c.is_ascii_hexdigit())
                && get_aot(orig, pos+5) == "'"
                { 6 } else { 0 },
        // ...lowercase u, signifying a unicode char code:
        "u" =>
            // Advance to the position after the closing single-quote, if valid.
            pos + detect_unicode_char_length(orig, pos, len),
        // ...anything else:
        _ =>
            // `pos` does not begin a char.
            pos
    }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, p: usize) -> &str { orig.get(p..p+1).unwrap_or("~") }

// 24-bit Unicode character code, 1 to 6 digits, eg '\u{f}' to '\u{10abCD}'.
fn detect_unicode_char_length(orig: &str, pos: usize, len: usize) -> usize {
    // If `orig` is not even long enough for the shortest form, '\u{0}',
    // or if the "'\u" is not followed by an open curly bracket, return zero.
    if len < pos + 7 || get_aot(orig, pos+3) != "{" { return 0 }
    // Initialise variables which will be modified by the loop, below.
    let mut found_closing_curly_bracket = false;
    let mut codepoint = "".to_string();
    // Loop through the characters after "'\u{", to a maximum "'\u{123456}".
    for i in 4..11 {
        let c = get_aot(orig, pos+i);
        if c == "}" { found_closing_curly_bracket = true; break }
        // If the current character is 0-9A-Fa-f, append it to `codepoint`.
        if c.chars().all(|c| c.is_ascii_hexdigit()) {
            codepoint.push_str(c)
        } else {
            return 0
        }
    }
    // Guard against an overlong unicode escape. Must have at most 6 hex digits.
    if ! found_closing_curly_bracket { return 0 }
    // Get the position of the character which should be a closing single-quote.
    let l = codepoint.len() + 5;
    // If that char is not a single-quote, return zero.
    if get_aot(orig, pos+l) != "'" { return 0 }
    // Parse the codepoint into a number.
    match u32::from_str_radix(&codepoint, 16) {
        // This error conditional is actually unreachable, because we used
        // `is_ascii_hexdigit()`, above.
        Err(_) => 0,
        // Unicode escapes must be at most 10FFFF. If it’s not above that,
        // return the position after the closing single-quote.
        Ok(value) => if value > 0x10FFFF { 0 } else { l + 1 },
    }
}


#[cfg(test)]
mod tests {
    use super::detect_character as detect;

    #[test]
    fn get_ascii_or_tilde() {
        // Test the logic of `get_aot()`.
        let orig = "abcd€f";
        assert_eq!(orig.get(0..0+1).unwrap_or("~"), "a");
        assert_eq!(orig.get(1..1+1).unwrap_or("~"), "b");
        assert_eq!(orig.get(4..4+1).unwrap_or("~"), "~"); // start of €
        assert_eq!(orig.get(5..5+1).unwrap_or("~"), "~"); // middle of €
        assert_eq!(orig.get(7..7+1).unwrap_or("~"), "f");
        assert_eq!(orig.get(8..8+1).unwrap_or("~"), "~"); // right on the end
        assert_eq!(orig.get(9..9+1).unwrap_or("~"), "~"); // past the end
    }

    #[test]
    fn detect_character_correct() {
        // Simple ascii char in the middle of other ascii text.
        let orig = "abcde'f'ghi";
        assert_eq!(detect(orig, 4), 4); // e'f
        assert_eq!(detect(orig, 5), 8); // 'f' advance three places
        assert_eq!(detect(orig, 6), 6); // f'g
        assert_eq!(detect(orig, 7), 7); // 'gh
        // Non-ascii chars in the middle of other non-ascii text.
        // //en.wikipedia.org/wiki/Thousand_Character_Classic
        let orig = "±'±'∆'∆'\u{10FFFF}'\u{10FFFF}'";
        assert_eq!(detect(orig, 0), 0); // ± is 2 bytes wide
        assert_eq!(detect(orig, 2), 6); // '±' advance four places
        assert_eq!(detect(orig, 6), 6); // ∆ is 3 bytes wide
        assert_eq!(detect(orig, 9), 14); // '∆' advance five places
        assert_eq!(detect(orig, 14), 14); // \u{10FFFF} is 4 bytes wide
        assert_eq!(detect(orig, 18), 24); // '\u{10FFFF}' advance five places
        // Simple backslash.
        let orig = " -'\\n'- ";
        assert_eq!(detect(orig, 1), 1); // -'\n
        assert_eq!(detect(orig, 2), 6); // '\n' advance four places
        assert_eq!(detect(orig, 3), 3); // \n'-
        assert_eq!(detect("'\\r'", 0), 4); // '\r'
        assert_eq!(detect("'\\t' ", 0), 4); // '\t'
        assert_eq!(detect("'\\\\'", 0), 4); // '\\'
        assert_eq!(detect(" '\\0'", 1), 5); // '\0'
        assert_eq!(detect("'\\\"'", 0), 4); // '\"'
        assert_eq!(detect("'\\''", 0), 4); // '\''
        // 7-bit '\x00'.
        let orig = "'\\x4A'";
        assert_eq!(detect(orig, 0), 6); // '\x4A' advance to end
        assert_eq!(detect(orig, 1), 1); // \x4A'
        assert_eq!(detect(orig, 5), 5); // '
        let orig = " - '\\x0f' - ";
        assert_eq!(detect(orig, 3), 9); // '\x0f' advance 6 places
        // Unicode '\u{0}'.
        assert_eq!(detect("'\\u{0}'", 0), 7); // '\u{0}'
        assert_eq!(detect(" '\\u{C}'", 1), 8); // '\u{C}'
        assert_eq!(detect("- '\\u{f}'", 2), 9); // '\u{f}'
        assert_eq!(detect("'\\u{00}'", 0), 8); // '\u{00}'
        assert_eq!(detect(" '\\u{bD}'", 1), 9); // '\u{bD}'
        assert_eq!(detect("'\\u{1cF}'", 0), 9); // '\u{1cF}'
        assert_eq!(detect("'\\u{fFfF}'", 0), 10); // '\u{fFfF}'
        assert_eq!(detect(" '\\u{00000}'", 1), 12); // '\u{00000}'
        assert_eq!(detect("'\\u{100abC}'", 0), 12); // '\u{100abC}'
        assert_eq!(detect(" - '\\u{10FFFF}'", 3), 15); // maximum
        assert_eq!(detect("'\\u{123}'€", 0), 9); // '\u{123}'
        let orig = "'\\u{30aF}'";
        assert_eq!(detect(orig, 0), 10); // '\u{30aF}' advance to end
        assert_eq!(detect(orig, 1), 1); // \u{30aF}'
        assert_eq!(detect(orig, 2), 2); // u{30aF}'
    }

    #[test]
    fn detect_character_incorrect() {
        // Empty.
        assert_eq!(detect("'' ", 0), 0); // '' missing char
        // Incorrect simple backslash.
        assert_eq!(detect("'\\' ", 0), 0); // '\' no char after the \
        assert_eq!(detect(" '\\\\", 1), 1); // '\\ has no end quote
        assert_eq!(detect("'\\q'", 0), 0); // '\q' no such backslash
        assert_eq!(detect("'\\~'", 0), 0); // '\~' no such backslash
        assert_eq!(detect(" '\\x'", 1), 1); // '\x' would start 7-bit
        assert_eq!(detect("'\\u'", 0), 0); // '\x' would start unicode
        // Incorrect 7-bit '\x00'.
        assert_eq!(detect("'\\x3' - ", 0), 0); // '\x3' has no 2nd digit
        assert_eq!(detect("'\\x3f - ", 0), 0); // '\x3f has no end quote
        assert_eq!(detect("'\\x0G'", 0), 0); // '\x0G' is not valid
        assert_eq!(detect("'\\x81'", 0), 0); // '\x81' is out of range
        // Incorrect Unicode '\u{0}'.
        assert_eq!(detect("'\\uxyz", 0), 0); // missing {0}
        assert_eq!(detect("'\\u{xyz", 0), 0); // missing 0}
        assert_eq!(detect("'\\u{0xyz", 0), 0); // missing }
        assert_eq!(detect("'\\u", 0), 0); // at end, missing {0}
        assert_eq!(detect("'\\u{", 0), 0); // at end, missing 0}
        assert_eq!(detect("'\\u{0", 0), 0); // at end, missing }
        assert_eq!(detect("'\\u[0]'", 0), 0); // square not curly
        assert_eq!(detect("'\\u{abcde", 0), 0); // missing }' at end
        assert_eq!(detect("'\\u{12i4}'", 0), 0); // not a hex digit
        assert_eq!(detect("'\\u{100abCd}'", 0), 0); // too long
        assert_eq!(detect("'\\u{1234}", 0), 0); // missing ' at end
        assert_eq!(detect("'\\u{1234} ", 0), 0); // no closing quote
        assert_eq!(detect("'\\u{110000}'", 0), 0); // too high
    }

    #[test]
    fn detect_character_will_not_panic() {
        // Near the end of `orig`.
        assert_eq!(detect("", 0), 0); // empty string
        assert_eq!(detect("'", 0), 0); // '
        assert_eq!(detect("'a", 0), 0); // 'a
        assert_eq!(detect("'\\", 0), 0); // '\
        assert_eq!(detect("'\\n", 0), 0); // '\n
        assert_eq!(detect("'\\x", 0), 0); // '\x
        assert_eq!(detect("'\\x4", 0), 0); // '\x4
        assert_eq!(detect("'\\x7f", 0), 0); // '\x7f
        assert_eq!(detect("'\\u", 0), 0); // '\u
        assert_eq!(detect("'\\u{", 0), 0); // '\u{
        assert_eq!(detect("'\\u{0", 0), 0); // '\u{0
        assert_eq!(detect("'\\u{0}", 0), 0); // '\u{0}
        assert_eq!(detect("'\\u{30aF", 0), 0); // '\u{30aF
        assert_eq!(detect("'\\u{30Af}", 0), 0); // '\u{30Af}
        // Invalid `pos`.
        assert_eq!(detect("abc", 2), 2); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3), 3); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4), 4); // 4 is out of range
        assert_eq!(detect("abc", 100), 100); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1), 1); // part way through the three eurobytes
        assert_eq!(detect("'€", 0), 0); // non-ascii after '
        assert_eq!(detect("'\\€", 0), 0); // non-ascii after '\
        assert_eq!(detect("'\\u€'", 0), 0); // non-ascii after '\u
        assert_eq!(detect("'\\u{€'", 0), 0); // non-ascii after '\u{
        assert_eq!(detect("'\\u{123€'", 0), 0); // non-ascii after '\u{123
        assert_eq!(detect("'\\u{123}€'", 0), 0); // non-ascii after '\u{123}
    }

}
