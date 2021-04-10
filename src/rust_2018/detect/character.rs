//! Detects a `char` literal, like `'A'` or `'\u{03aB}'`.

use super::super::lexeme::LexemeKind;
const HEX:  LexemeKind = LexemeKind::CharacterHex;
const PLAIN:  LexemeKind = LexemeKind::CharacterPlain;
const UNICODE:  LexemeKind = LexemeKind::CharacterUnicode;
const UNDETECTED: (LexemeKind, usize) = (LexemeKind::Undetected, 0);

/// Detects a `char` literal, like `'A'` or `'\u{03aB}'`.
/// 
/// @TODO `b` prefix, eg `b'A'`
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `chr` The character position in `orig` to look at
/// 
/// ### Returns
/// If `chr` begins a valid looking char literal, `detect_character()` returns
/// the appropriate `LexemeKind::Character*` and the position after it ends.  
/// Otherwise, `detect_character()` returns `LexemeKind::Undetected` and `0`.
pub fn detect_character(
    orig: &str,
    chr: usize,
) -> (
    LexemeKind,
    usize,
) {
    // Avoid panicking, if there would not be enough room for a char.
    let len = orig.len();
    if len < chr + 3 { return UNDETECTED } // chr + ' + A + '
    // If the current char is not a single-quote, then it does not begin a char.
    let c0 = get_aot(orig, chr);
    if c0 != "'" { return UNDETECTED }
    // Get the next char, even if it’s not ascii.
    let mut c1_end = chr + 2;
    while !orig.is_char_boundary(c1_end) { c1_end += 1 }
    // Avoid panicking, if there would not be enough room for a char.
    if len < c1_end + 1 { return UNDETECTED }
    let c1 = &orig[chr+1..c1_end];
    // If the next char is not a backslash:
    if c1 != "\\" {
        return
            // If `c1` is a single quote:
            if c1 == "'"
                // We have found the string "''", which is not a valid char.
                { UNDETECTED }
            // Otherwise, if the char directly after `c1` is not a single quote:
            else if get_aot(orig, c1_end) != "'"
                // We have probably found a label, like "'static".
                { UNDETECTED }
            // Otherwise, this is a valid char literal, like "'A'" or "'±'".
            else { (PLAIN, c1_end + 1) }
    }

    // Now we know `c1` is a backslash, if the char after it is...
    match get_aot(orig, chr+2) {
        // ...one of Rust’s simple backslashable chars:
        "n" | "r" | "t" | "\\" | "0" | "\"" | "'" =>
            // Advance four places if the char after that is a single-quote.
            if len >= chr + 4
            && get_aot(orig, chr+3) == "'"
                { (PLAIN, chr + 4) } else { UNDETECTED },
        // ...lowercase x, signifying a 7-bit char code:
        "x" =>
            // Advance 6 places if the chars after that are 0-7 and 0-9A-Fa-f.
            if len >= chr + 6
            && get_aot(orig, chr+3).chars().all(|c| c >= '0' && c <= '7')
            && get_aot(orig, chr+4).chars().all(|c| c.is_ascii_hexdigit())
            && get_aot(orig, chr+5) == "'"
                { (HEX, chr + 6) } else { UNDETECTED },
        // ...lowercase u, signifying a unicode char code:
        "u" =>
            // Advance to the position after the closing single-quote, if valid.
            detect_unicode_char(orig, chr, len),
        // ...anything else:
        _ =>
            // `chr` does not begin a char.
            UNDETECTED
    }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, c: usize) -> &str { orig.get(c..c+1).unwrap_or("~") }

// 24-bit Unicode character code, 1 to 6 digits, eg '\u{f}' to '\u{10abCD}'.
fn detect_unicode_char(
    orig: &str,
    chr: usize,
    len: usize,
) -> (
    LexemeKind,
    usize,
) {
    // If `orig` is not even long enough for the shortest form, '\u{0}', or if
    // the "'\u" is not followed by an open curly bracket, this is not a char.
    if len < chr + 7 || get_aot(orig, chr+3) != "{" { return UNDETECTED }
    // Initialise variables which will be modified by the loop, below.
    let mut found_closing_curly_bracket = false;
    let mut codepoint = "".to_string();
    // Loop through the characters after "'\u{", to a maximum "'\u{123456}".
    for i in 4..11 {
        let c = get_aot(orig, chr+i);
        if c == "}" { found_closing_curly_bracket = true; break }
        // If the current character is 0-9A-Fa-f, append it to `codepoint`.
        if c.chars().all(|c| c.is_ascii_hexdigit()) {
            codepoint.push_str(c)
        } else {
            return UNDETECTED
        }
    }
    // Guard against an overlong unicode escape. Must have at most 6 hex digits.
    if ! found_closing_curly_bracket { return UNDETECTED }
    // Get the position of the character which should be a closing single-quote.
    let l = codepoint.len() + 5;
    // If that char is not a single-quote, this is not a char.
    if get_aot(orig, chr+l) != "'" { return UNDETECTED }
    // Parse the codepoint into a number.
    match u32::from_str_radix(&codepoint, 16) {
        // This error conditional is actually unreachable, because we used
        // `is_ascii_hexdigit()`, above.
        Err(_) => UNDETECTED,
        // Unicode escapes must be at most 10FFFF. If it’s not above that,
        // return the position after the closing single-quote.
        Ok(value) =>
            if value > 0x10FFFF { UNDETECTED } else { (UNICODE, chr + l + 1) },
    }
}


#[cfg(test)]
mod tests {
    use super::detect_character as detect;
    use super::HEX as H;
    use super::PLAIN as P;
    use super::UNICODE as C;
    use super::UNDETECTED as U;

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
        assert_eq!(detect(orig, 4),  U);    // e'f
        assert_eq!(detect(orig, 5), (P,8)); // 'f' advance three places
        assert_eq!(detect(orig, 6),  U);    // f'g
        assert_eq!(detect(orig, 7),  U);    // 'gh
        // Non-ascii chars in the middle of other non-ascii text.
        // //en.wikipedia.org/wiki/Thousand_Character_Classic
        let orig = "±'±'∆'∆'\u{10FFFF}'\u{10FFFF}'";
        assert_eq!(detect(orig, 0),   U);     // ± is 2 bytes wide
        assert_eq!(detect(orig, 2),  (P,6));  // '±' advance four places
        assert_eq!(detect(orig, 6),   U);     // ∆ is 3 bytes wide
        assert_eq!(detect(orig, 9),  (P,14)); // '∆' advance five places
        assert_eq!(detect(orig, 14),  U);     // \u{10FFFF} is 4 bytes wide
        assert_eq!(detect(orig, 18), (P,24)); // '\u{10FFFF}' advance 5 places
        // Simple backslash.
        let orig = " -'\\n'- ";
        assert_eq!(detect(orig, 1),      U);    // -'\n
        assert_eq!(detect(orig, 2),     (P,6)); // '\n' advance four places
        assert_eq!(detect(orig, 3),      U);    // \n'-
        assert_eq!(detect("'\\r'", 0),  (P,4)); // '\r'
        assert_eq!(detect("'\\t' ", 0), (P,4)); // '\t'
        assert_eq!(detect("'\\\\'", 0), (P,4)); // '\\'
        assert_eq!(detect(" '\\0'", 1), (P,5)); // '\0'
        assert_eq!(detect("'\\\"'", 0), (P,4)); // '\"'
        assert_eq!(detect("'\\''", 0),  (P,4)); // '\''
        // 7-bit '\x00'.
        let orig = "'\\x4A'";
        assert_eq!(detect(orig, 0), (H,6)); // '\x4A' advance to end
        assert_eq!(detect(orig, 1),  U);    // \x4A'
        assert_eq!(detect(orig, 5),  U);    // '
        let orig = " - '\\x0f' - ";
        assert_eq!(detect(orig, 3), (H,9)); // '\x0f' advance 6 places
        // Unicode '\u{0}'.
        assert_eq!(detect("'\\u{0}'",         0), (C,7));  // '\u{0}'
        assert_eq!(detect(" '\\u{C}'",        1), (C,8));  // '\u{C}'
        assert_eq!(detect("- '\\u{f}'",       2), (C,9));  // '\u{f}'
        assert_eq!(detect("'\\u{00}'",        0), (C,8));  // '\u{00}'
        assert_eq!(detect(" '\\u{bD}'",       1), (C,9));  // '\u{bD}'
        assert_eq!(detect("'\\u{1cF}'",       0), (C,9));  // '\u{1cF}'
        assert_eq!(detect("'\\u{fFfF}'",      0), (C,10)); // '\u{fFfF}'
        assert_eq!(detect(" '\\u{00000}'",    1), (C,12)); // '\u{00000}'
        assert_eq!(detect("'\\u{100abC}'",    0), (C,12)); // '\u{100abC}'
        assert_eq!(detect(" - '\\u{10FFFF}'", 3), (C,15)); // maximum
        assert_eq!(detect("'\\u{123}'€",      0), (C,9));  // '\u{123}'
        let orig = "'\\u{30aF}'";
        assert_eq!(detect(orig, 0), (C,10)); // '\u{30aF}' advance to end
        assert_eq!(detect(orig, 1),  U);     // \u{30aF}'
        assert_eq!(detect(orig, 2),  U);     // u{30aF}'
    }

    #[test]
    fn detect_character_incorrect() {
        // Empty.
        assert_eq!(detect("'' ", 0), U); // '' missing char
        // Incorrect simple backslash.
        assert_eq!(detect("'\\' ", 0),  U); // '\' no char after the \
        assert_eq!(detect(" '\\\\", 1), U); // '\\ has no end quote
        assert_eq!(detect("'\\q'", 0),  U); // '\q' no such backslash
        assert_eq!(detect("'\\~'", 0),  U); // '\~' no such backslash
        assert_eq!(detect(" '\\x'", 1), U); // '\x' would start 7-bit
        assert_eq!(detect("'\\u'", 0),  U); // '\x' would start unicode
        // Incorrect 7-bit '\x00'.
        assert_eq!(detect("'\\x3' - ", 0), U); // '\x3' has no 2nd digit
        assert_eq!(detect("'\\x3f - ", 0), U); // '\x3f has no end quote
        assert_eq!(detect("'\\x0G'", 0),   U); // '\x0G' is not valid
        assert_eq!(detect("'\\x81'", 0),   U); // '\x81' is out of range
        // Incorrect Unicode '\u{0}'.
        assert_eq!(detect("'\\uxyz", 0), U); // missing {0}
        assert_eq!(detect("'\\u{xyz", 0), U); // missing 0}
        assert_eq!(detect("'\\u{0xyz", 0), U); // missing }
        assert_eq!(detect("'\\u", 0), U); // at end, missing {0}
        assert_eq!(detect("'\\u{", 0), U); // at end, missing 0}
        assert_eq!(detect("'\\u{0", 0), U); // at end, missing }
        assert_eq!(detect("'\\u[0]'", 0), U); // square not curly
        assert_eq!(detect("'\\u{abcde", 0), U); // missing }' at end
        assert_eq!(detect("'\\u{12i4}'", 0), U); // not a hex digit
        assert_eq!(detect("'\\u{100abCd}'", 0), U); // too long
        assert_eq!(detect("'\\u{1234}", 0), U); // missing ' at end
        assert_eq!(detect("'\\u{1234} ", 0), U); // no closing quote
        assert_eq!(detect("'\\u{110000}'", 0), U); // too high
    }

    #[test]
    fn detect_character_will_not_panic() {
        // Near the end of `orig`.
        assert_eq!(detect("", 0), U); // empty string
        assert_eq!(detect("'", 0), U); // '
        assert_eq!(detect("'a", 0), U); // 'a
        assert_eq!(detect("'\\", 0), U); // '\
        assert_eq!(detect("'\\n", 0), U); // '\n
        assert_eq!(detect("'\\x", 0), U); // '\x
        assert_eq!(detect("'\\x4", 0), U); // '\x4
        assert_eq!(detect("'\\x7f", 0), U); // '\x7f
        assert_eq!(detect("'\\u", 0), U); // '\u
        assert_eq!(detect("'\\u{", 0), U); // '\u{
        assert_eq!(detect("'\\u{0", 0), U); // '\u{0
        assert_eq!(detect("'\\u{0}", 0), U); // '\u{0}
        assert_eq!(detect("'\\u{30aF", 0), U); // '\u{30aF
        assert_eq!(detect("'\\u{30Af}", 0), U); // '\u{30Af}
        // Invalid `chr`.
        assert_eq!(detect("abc", 2),   U); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3),   U); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4),   U); // 4 is out of range
        assert_eq!(detect("abc", 100), U); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1), U); // part way into the three € bytes
        assert_eq!(detect("'€", 0), U); // non-ascii after '
        assert_eq!(detect("'\\€", 0), U); // non-ascii after '\
        assert_eq!(detect("'\\u€'", 0), U); // non-ascii after '\u
        assert_eq!(detect("'\\u{€'", 0), U); // non-ascii after '\u{
        assert_eq!(detect("'\\u{123€'", 0), U); // non-ascii after '\u{123
        assert_eq!(detect("'\\u{123}€'", 0), U); // non-ascii after '\u{123}
    }

}
