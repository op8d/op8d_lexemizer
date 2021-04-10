//! Detects a string literal, like `"Hello \"Rust\""` or `r#"Hello "Rust""#`.

use super::super::lexeme::LexemeKind;
const PLAIN:  LexemeKind = LexemeKind::StringPlain;
const RAW: LexemeKind = LexemeKind::StringRaw;
const UNDETECTED: (LexemeKind, usize) = (LexemeKind::Undetected, 0);

/// Detects a string literal, like `"Hello \"Rust\""` or `r#"Hello "Rust""#`.
/// 
/// @TODO `b` prefix, eg `b"Just the bytes"`
/// @TODO `br` prefix, eg `br#"Just "the" bytes"#`
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `chr` The character position in `orig` to look at
/// 
/// ### Returns
/// If `chr` begins a valid looking string literal, `detect_string()` returns
/// the appropriate `LexemeKind::String*` and the position after it ends.  
/// Otherwise, `detect_string()` returns `LexemeKind::Undetected` and `0`.
pub fn detect_string(
    orig: &str,
    chr: usize,
) -> (
    LexemeKind,
    usize,
) {
    // If the current char is the last in `orig`, it does not begin a string.
    let len = orig.len();
    if len < chr + 1 { return UNDETECTED }

    // If the current char is:
    match get_aot(orig, chr) {
        // A double quote, `chr` could begin a Plain string.
        "\"" => detect_plain_string(orig, chr, len),
        // A lowercase "r", `chr` could begin a Raw string.
        "r" => detect_raw_string(orig, chr, len),
        // Anything else, `chr` does not begin a string.
        _ => UNDETECTED,
    }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, c: usize) -> &str { orig.get(c..c+1).unwrap_or("~") }

fn detect_plain_string(
    orig: &str,
    chr: usize,
    len: usize,
) -> (
    LexemeKind,
    usize,
) {
    // Slightly hacky way to to skip forward while looping.
    let mut i = chr + 1;
    // Step through each char, from `chr` to the end of the original input code.
    while i < len {
        // Get this character, even if it’s non-ascii.
        let mut j = i + 1;
        while !orig.is_char_boundary(j) { j += 1 }
        let c = &orig[i..j];
        // If this char is a backslash:
        if c == "\\" {
            // If the backlash ends the input code, this is not a string.
            if j == len { return UNDETECTED }
            // Ignore the next character, even if it’s non-ascii.
            // Treat "\€" as a string Lexeme, even though it’s invalid code.
            j += 1;
            while !orig.is_char_boundary(j) { j += 1 }
        // If this char is a double quote:
        } else if c == "\"" {
            // Advance to the end of the double quote.
            return (PLAIN, j)
        }
        // Step forward, ready for the next iteration.
        i = j;
    }
    // The closing double quote was not found, so this is not a string.
    UNDETECTED
}

// doc.rust-lang.org/reference/tokens.html#raw-string-literals
fn detect_raw_string(
    orig: &str,
    chr: usize,
    len: usize,
) -> (
    LexemeKind,
    usize,
) {
    // If there are less than two chars after the "r", it cannot begin a string.
    if len < chr + 3 { return UNDETECTED }
    // Slightly hacky way to to skip forward while looping.
    let mut i = chr + 1;
    // Keep track of the number of leading hashes.
    let mut hashes = 0;
    // Keep track of finding the opening and closing double quotes.
    let mut found_opening_dq = false;
    let mut found_closing_dq = false;

    // Step through each char, from `chr` to the end of the original input code.
    // `len-1` saves a nanosecond or two, but also prevents `orig[i..i+1]` from
    // panicking at the end of the input.
    while i < len {
        // Get this character, even if it’s non-ascii.
        let mut j = i + 1;
        while !orig.is_char_boundary(j) { j += 1 }
        let c = &orig[i..j];

        // If we have not found the opening double quote yet:
        if ! found_opening_dq {
            // If this is the opening double quote, note that it’s been found.
            if c == "\"" {
                found_opening_dq = true
            // Otherwise, if this is a leading hash, increment the tally.
            } else if c == "#" {
                hashes += 1
            // Anything else is not valid for the start of a Raw string.
            } else {
                return UNDETECTED
            }

        // Otherwise, if we have already found the closing double quote:
        } else if found_closing_dq {
            // If we are not expecting any more hashes:
            if hashes == 0 {
                // Valid Raw string, advance to the end of the double quote.
                return (RAW, j)
            // Otherwise, if this is a trailing hash, decrement the tally.
            } else if c == "#" {
                hashes -= 1;
                // If we are not expecting any more hashes:
                if hashes == 0 {
                    // Valid Raw string, advance to the end of the double quote.
                    return (RAW, j)
                }
            // Anything else is not valid for the end of a Raw string.
            } else {
                return UNDETECTED
            }

        // Otherwise we are inside the main part of the string:
        } else {
            // If this char is a backslash:
            if c == "\\" {
                // If the backlash ends the input code, this is not a string.
                if j == len { return UNDETECTED }
                // Ignore the next character, even if it’s non-ascii.
                // Treat "\€" as a string Lexeme, even though it’s invalid code.
                j += 1;
                while !orig.is_char_boundary(j) { j += 1 }
            // If this char is a double quote:
            } else if c == "\"" {
                // Note that the closing double quote has been found.
                found_closing_dq = true;
                // If we are not expecting any more hashes:
                if hashes == 0 {
                    // Valid Raw string, advance to the end of the double quote.
                    return (RAW, j)
                }
            }
        }

        // Step forward, ready for the next iteration.
        i = j;
    }

    // Reached the end of the `orig` input string. Any leading hashes should
    // have been balanced by trailing hashes.
    if found_closing_dq && hashes == 0 { (RAW, i) } else { UNDETECTED }
}


#[cfg(test)]
mod tests {
    use super::detect_string as detect;
    use super::PLAIN as P;
    use super::RAW as R;
    use super::UNDETECTED as U;

    #[test]
    fn detect_string_correct() {
        // Plain.
        let orig = "abc\"ok\"xyz";
        assert_eq!(detect(orig, 2),  U);    // c"ok
        assert_eq!(detect(orig, 3), (P,7)); // "ok" advance four places
        assert_eq!(detect(orig, 4),  U);    // ok"x
        // Raw.
        assert_eq!(detect("-r\"ok\"-", 1), (R,6));
        assert_eq!(detect("r#\"ok\"#", 0), (R,7));
        assert_eq!(detect("abcr###\"ok\"###xyz", 3), (R,14));
        assert_eq!(detect("abcr###\"ok\"####xyz", 3), (R,14));
        // Byte.
        // @TODO
        // Byte raw.
        // @TODO

        // Escapes.
        // Escaped double quote.
        let orig = "a\"b\\\"c\"d";
        assert_eq!(detect(orig, 0),  U);    // a"b\"c
        assert_eq!(detect(orig, 1), (P,7)); // "b\"c" advance six places
        assert_eq!(detect(orig, 2),  U);    // b\"c"d
        assert_eq!(detect(orig, 3),  U);    // \"c"d
        assert_eq!(detect(orig, 4), (P,7)); // "c"d no ‘lookbehind’ happens!
        // Correct escapes, Plain string.
        let orig = r#"a"\0\\\\\"\\\n"z"#;
        assert_eq!(detect(orig, 0),   U);     // a"\0\\\\\"\\\n"
        assert_eq!(detect(orig, 1),  (P,15)); // "\0\\\\\"\\\n"z
        assert_eq!(detect(orig, 2),   U);     // \0\\\\\"\\\n"z
        assert_eq!(detect(orig, 9),  (P,15)); // "\\\n"z no ‘lookbehind’s!
        assert_eq!(detect(orig, 14),  U);     // "z not a string, has no end
        // Correct escapes, Raw string.
        assert_eq!(detect("r\"\\0\\n\\t\"", 0), (R,9)); // r"\0\n\t"
    }

    #[test]
    fn detect_string_incorrect() {
        // Incorrect escapes, Plain string.
        assert_eq!(detect("\\a\\b\\c", 0), U); // \a\b\c
        // Incorrect escapes, Raw string.
        assert_eq!(detect("r#\"\\X\\Y\\Z\"#", 0), (R,11)); // r#"\X\Y\Z"#
        // Incorrect raw.
        assert_eq!(detect("r##X#\" X in leading hashes \"###", 0), U);
        assert_eq!(detect("r###\" X in trailing hashes \"##X#", 0), U);
        assert_eq!(detect("r###\" too few trailing hashes \"##", 0), U);
        assert_eq!(detect("-r###\" no trailing hashes \"-", 1), U);
        // Incorrect byte.
        // @TODO
        // Incorrect byte raw.
        // @TODO
    }

    #[test]
    fn detect_string_will_not_panic() {
        // Near the end of the `orig` input code.
        assert_eq!(detect("", 0), U);                   // empty string
        assert_eq!(detect("\"", 0), U);                 // "
        assert_eq!(detect("\"a", 0), U);                // "a
        assert_eq!(detect("\"\\", 0), U);               // "\
        assert_eq!(detect("\"\\n", 0), U);              // "\n
        assert_eq!(detect("\"\\z", 0), U);              // "\z
        assert_eq!(detect("\"\\z\\", 0), U);            // "\z\
        assert_eq!(detect("\"\\z\\\"", 0), U);          // "\z\"
        assert_eq!(detect("r", 0), U);                  // r
        assert_eq!(detect("r\"", 0), U);                // r"
        assert_eq!(detect("r\"a", 0), U);               // r"a
        assert_eq!(detect("r\"\\", 0), U);              // r"\
        assert_eq!(detect("r\"\\n", 0), U);             // r"\n
        assert_eq!(detect("r\"\\z", 0), U);             // r"\z
        assert_eq!(detect("r\"\\z\\", 0), U);           // r"\z\
        assert_eq!(detect("r\"\\z\\\"", 0), U);         // r"\z\"
        assert_eq!(detect("r\"\\z\\\"\"", 0), (R,7));   // r"\z\""
        assert_eq!(detect("r#", 0), U);                 // r#
        assert_eq!(detect("r#\"", 0), U);               // r#"
        assert_eq!(detect("r#\"a", 0), U);              // r#"a
        assert_eq!(detect("r#\"\\", 0), U);             // r#"\
        assert_eq!(detect("r#\"\\n", 0), U);            // r#"\n
        assert_eq!(detect("r#\"\\z", 0), U);            // r#"\z
        assert_eq!(detect("r#\"\\z\\", 0), U);          // r#"\z\
        assert_eq!(detect("r#\"\\z\\\"", 0), U);        // r#"\z\"
        assert_eq!(detect("r#\"\\z\\\"#", 0), U);       // r#"\z\"#
        assert_eq!(detect("r#\"\\z\\\"\"#", 0), (R,9)); // r#"\z\""#
        assert_eq!(detect("r##\"\\z\\\"\"#", 0), U);    // r##"\z\""# missing #
        // Invalid `chr`.
        assert_eq!(detect("abc", 2), U);   // 2 is before "c", so in range
        assert_eq!(detect("abc", 3), U);   // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4), U);   // 4 is out of range
        assert_eq!(detect("abc", 100), U); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1), U); // part way into the three € bytes
        assert_eq!(detect("\"€", 0), U); // non-ascii after "
        assert_eq!(detect("\"a€", 0), U); // non-ascii after "a
        assert_eq!(detect("\"\\€", 0), U); // non-ascii after "\
        assert_eq!(detect("\"\\z€", 0), U); // non-ascii after "\z
        assert_eq!(detect("\"\\z\\€", 0), U); // non-ascii after "\z\
        assert_eq!(detect("\"\\z\\\"€", 0), U); // non-ascii after "\z\"
        assert_eq!(detect("\"\\z\\\"\"€", 0), (P,6)); // non-ascii after "\z\""
        assert_eq!(detect("\"€\"", 0), (P,5)); // three-byte non-ascii in ""
        assert_eq!(detect("\"a€\"", 0), (P,6)); // non-ascii in "a"
        assert_eq!(detect("\"\\€\"", 0), (P,6)); // non-ascii in "\"
        assert_eq!(detect("\"\\z€\"", 0), (P,7)); // non-ascii in "\z"
        assert_eq!(detect("\"\\z\\€\"", 0), (P,8)); // non-ascii in "\z\"
        assert_eq!(detect("r\"€", 0), U); // non-ascii after r"
        assert_eq!(detect("r\"a€", 0), U); // non-ascii after r"a
        assert_eq!(detect("r\"\\€", 0), U); // non-ascii after r"\
        assert_eq!(detect("r\"\\z€", 0), U); // non-ascii after r"\z
        assert_eq!(detect("r\"\\z\\€", 0), U); // non-ascii after r"\z\
        assert_eq!(detect("r\"\\z\\\"€", 0), U); // non-ascii after r"\z\"
        assert_eq!(detect("r\"\\z\\\"\"€", 0), (R,7)); // non-ascii after r"\z\""
        assert_eq!(detect("r\"€\"", 0), (R,6)); // non-ascii in r""
        assert_eq!(detect("r\"a€\"", 0), (R,7)); // non-ascii in r"a"
        assert_eq!(detect("r\"\\€\"", 0), (R,7)); // non-ascii in r"\"
        assert_eq!(detect("r\"\\z€\"", 0), (R,8)); // non-ascii in r"\z"
        assert_eq!(detect("r\"\\z\\€\"", 0), (R,9)); // non-ascii in r"\z\"
        assert_eq!(detect("r#\"€", 0), U); // non-ascii after r#"
        assert_eq!(detect("r#\"a€", 0), U); // non-ascii after r#"a
        assert_eq!(detect("r#\"\\€", 0), U); // non-ascii after r#"\
        assert_eq!(detect("r#\"\\z€", 0), U); // non-ascii after r#"\z
        assert_eq!(detect("r#\"\\z\\€", 0), U); // non-ascii after r#"\z\
        assert_eq!(detect("r#\"\\z\\\"€", 0), U); // non-ascii after r#"\z\"
        assert_eq!(detect("r#\"\\z\"€", 0), U); // non-ascii after r#"\z"
        assert_eq!(detect("r#\"€\"", 0), U); // non-ascii in r#""
        assert_eq!(detect("r#\"a€\"", 0), U); // non-ascii in r#"a"
        assert_eq!(detect("r#\"\\€\"", 0), U); // non-ascii in r#"\"
        assert_eq!(detect("r#\"\\z€\"", 0), U); // non-ascii in r#"\z"
        assert_eq!(detect("r#\"\\z\\€\"", 0), U); // non-ascii in r#"\z\"
        assert_eq!(detect("r#\"\\z\\€\\\"\"#", 0), (R,13)); // r#"\z\€\""#
        assert_eq!(detect("r##\"\\z\\€\\\"\"#", 0), U); // missing hash at end
    }

}