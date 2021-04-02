//! Detects a string literal, like `"Hello \"Rust\""` or `r#"Hello "Rust""#`.

/// Detects a string literal, like `"Hello \"Rust\""` or `r#"Hello "Rust""#`.
/// 
/// @TODO `b` prefix, eg `b"Just the bytes"`
/// @TODO `br` prefix, eg `br#"Just "the" bytes"#`
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `pos` The character position in `orig` to look at
/// 
/// ### Returns
/// If `pos` begins a valid looking string literal, `detect_string()` returns
/// the character position after the closing single quote (or hash).  
/// Otherwise, `detect_string()` just returns the `pos` argument.
pub fn detect_string(orig: &str, pos: usize) -> usize {
    // If the current char is the last in `orig`, it does not begin a string.
    let len = orig.len();
    if len < pos + 1 { return pos }

    // If the current char is:
    match get_aot(orig, pos) {
        // A double quote, `pos` could begin a regular string.
        "\"" => detect_regular_string(orig, pos, len),
        // A lowercase "r", `pos` could begin a raw string.
        "r" => detect_raw_string(orig, pos, len),
        // Anything else, `pos` does not begin a string.
        _ => pos,
    }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, p: usize) -> &str { orig.get(p..p+1).unwrap_or("~") }

fn detect_regular_string(orig: &str, pos: usize, len: usize) -> usize {
    // Slightly hacky way to to skip forward while looping.
    let mut i = pos + 1;
    // Step through each char, from `pos` to the end of the original input code.
    while i < len {
        // Get this character, even if it’s non-ascii.
        let mut j = i + 1;
        while !orig.is_char_boundary(j) { j += 1 }
        let c = &orig[i..j];
        // If this char is a backslash:
        if c == "\\" {
            // If the backlash ends the input code, this is not a string.
            if j == len { return pos }
            // Ignore the next character, even if it’s non-ascii.
            // Treat "\€" as a string Lexeme, even though it’s invalid code.
            j += 1;
            while !orig.is_char_boundary(j) { j += 1 }
        // If this char is a double quote:
        } else if c == "\"" {
            // Advance to the end of the double quote.
            return j
        }
        // Step forward, ready for the next iteration.
        i = j;
    }
    // The closing double quote was not found, so this is not a string.
    pos
}

// doc.rust-lang.org/reference/tokens.html#raw-string-literals
fn detect_raw_string(orig: &str, pos: usize, len: usize) -> usize {
    // If there are less than two chars after the "r", it cannot begin a string.
    if len < pos + 3 { return pos }
    // Slightly hacky way to to skip forward while looping.
    let mut i = pos + 1;
    // Keep track of the number of leading hashes.
    let mut hashes = 0;
    // Keep track of finding the opening and closing double quotes.
    let mut found_opening_dq = false;
    let mut found_closing_dq = false;

    // Step through each char, from `pos` to the end of the original input code.
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
            // Anything else is not valid for the start of a raw string.
            } else {
                return pos
            }

        // Otherwise, if we have already found the closing double quote:
        } else if found_closing_dq {
            // If we are not expecting any more hashes:
            if hashes == 0 {
                // Valid raw string, advance to the end of the double quote.
                return j
            // Otherwise, if this is a trailing hash, decrement the tally.
            } else if c == "#" {
                hashes -= 1;
                // If we are not expecting any more hashes:
                if hashes == 0 {
                    // Valid raw string, advance to the end of the double quote.
                    return j
                }
            // Anything else is not valid for the end of a raw string.
            } else {
                return pos
            }

        // Otherwise we are inside the main part of the string:
        } else {
            // If this char is a backslash:
            if c == "\\" {
                // If the backlash ends the input code, this is not a string.
                if j == len { return pos }
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
                    // Valid raw string, advance to the end of the double quote.
                    return j
                }
            }
        }

        // Step forward, ready for the next iteration.
        i = j;
    }

    // Reached the end of the `orig` input string. Any leading hashes should have
    // been balanced by trailing hashes.
    if found_closing_dq && hashes == 0 { i } else { pos }
}


#[cfg(test)]
mod tests {
    use super::detect_string as detect;
    

    #[test]
    fn detect_string_correct() {
        // Regular.
        let orig = "abc\"ok\"xyz";
        assert_eq!(detect(orig, 2), 2); // c"ok
        assert_eq!(detect(orig, 3), 7); // "ok" advance four places
        assert_eq!(detect(orig, 4), 4); // ok"x
        // Raw.
        assert_eq!(detect("-r\"ok\"-", 1), 6);
        assert_eq!(detect("r#\"ok\"#", 0), 7);
        assert_eq!(detect("abcr###\"ok\"###xyz", 3), 14);
        assert_eq!(detect("abcr###\"ok\"####xyz", 3), 14);
        // Byte.
        // @TODO
        // Byte raw.
        // @TODO

        // Escapes.
        // Escaped double quote.
        let orig = "a\"b\\\"c\"d";
        assert_eq!(detect(orig, 0), 0); // a"b\"c
        assert_eq!(detect(orig, 1), 7); // "b\"c" advance six places
        assert_eq!(detect(orig, 2), 2); // b\"c"d
        assert_eq!(detect(orig, 3), 3); // \"c"d
        assert_eq!(detect(orig, 4), 7); // "c"d no ‘lookbehind’ happens!
        // Correct escapes, regular string.
        let orig = r#"a"\0\\\\\"\\\n"z"#;
        assert_eq!(detect(orig, 0),  0);  // a"\0\\\\\"\\\n"
        assert_eq!(detect(orig, 1),  15); // "\0\\\\\"\\\n"z
        assert_eq!(detect(orig, 2),  2);  // \0\\\\\"\\\n"z
        assert_eq!(detect(orig, 9),  15); // "\\\n"z no ‘lookbehind’s!
        assert_eq!(detect(orig, 14), 14); // "z not a string, has no end
        // Correct escapes, raw string.
        assert_eq!(detect("r\"\\0\\n\\t\"", 0), 9); // r"\0\n\t"
    }

    #[test]
    fn detect_string_incorrect() {
        // Incorrect escapes, regular string.
        assert_eq!(detect("\\a\\b\\c", 0), 0); // \a\b\c
        // Incorrect escapes, raw string.
        assert_eq!(detect("r#\"\\X\\Y\\Z\"#", 0), 11); // r#"\X\Y\Z"#
        // Incorrect raw.
        assert_eq!(detect("r##X#\" X in leading hashes \"###", 0), 0);
        assert_eq!(detect("r###\" X in trailing hashes \"##X#", 0), 0);
        assert_eq!(detect("r###\" too few trailing hashes \"##", 0), 0);
        assert_eq!(detect("-r###\" no trailing hashes \"-", 1), 1);
        // Incorrect byte.
        // @TODO
        // Incorrect byte raw.
        // @TODO
    }

    #[test]
    fn detect_string_will_not_panic() {
        // Near the end of the `orig` input code.
        assert_eq!(detect("", 0), 0);               // empty string
        assert_eq!(detect("\"", 0), 0);             // "
        assert_eq!(detect("\"a", 0), 0);            // "a
        assert_eq!(detect("\"\\", 0), 0);           // "\
        assert_eq!(detect("\"\\n", 0), 0);          // "\n
        assert_eq!(detect("\"\\z", 0), 0);          // "\z
        assert_eq!(detect("\"\\z\\", 0), 0);        // "\z\
        assert_eq!(detect("\"\\z\\\"", 0), 0);      // "\z\"
        assert_eq!(detect("r", 0), 0);              // r
        assert_eq!(detect("r\"", 0), 0);            // r"
        assert_eq!(detect("r\"a", 0), 0);           // r"a
        assert_eq!(detect("r\"\\", 0), 0);          // r"\
        assert_eq!(detect("r\"\\n", 0), 0);         // r"\n
        assert_eq!(detect("r\"\\z", 0), 0);         // r"\z
        assert_eq!(detect("r\"\\z\\", 0), 0);       // r"\z\
        assert_eq!(detect("r\"\\z\\\"", 0), 0);     // r"\z\"
        assert_eq!(detect("r\"\\z\\\"\"", 0), 7);   // r"\z\""
        assert_eq!(detect("r#", 0), 0);             // r#
        assert_eq!(detect("r#\"", 0), 0);           // r#"
        assert_eq!(detect("r#\"a", 0), 0);          // r#"a
        assert_eq!(detect("r#\"\\", 0), 0);         // r#"\
        assert_eq!(detect("r#\"\\n", 0), 0);        // r#"\n
        assert_eq!(detect("r#\"\\z", 0), 0);        // r#"\z
        assert_eq!(detect("r#\"\\z\\", 0), 0);      // r#"\z\
        assert_eq!(detect("r#\"\\z\\\"", 0), 0);    // r#"\z\"
        assert_eq!(detect("r#\"\\z\\\"#", 0), 0);   // r#"\z\"#
        assert_eq!(detect("r#\"\\z\\\"\"#", 0), 9); // r#"\z\""#
        assert_eq!(detect("r##\"\\z\\\"\"#", 0), 0);// r##"\z\""# missing hash
        // Invalid `pos`.
        assert_eq!(detect("abc", 2), 2); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3), 3); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4), 4); // 4 is out of range
        assert_eq!(detect("abc", 100), 100); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1), 1); // part way through the three eurobytes
        assert_eq!(detect("\"€", 0), 0); // non-ascii after "
        assert_eq!(detect("\"a€", 0), 0); // non-ascii after "a
        assert_eq!(detect("\"\\€", 0), 0); // non-ascii after "\
        assert_eq!(detect("\"\\z€", 0), 0); // non-ascii after "\z
        assert_eq!(detect("\"\\z\\€", 0), 0); // non-ascii after "\z\
        assert_eq!(detect("\"\\z\\\"€", 0), 0); // non-ascii after "\z\"
        assert_eq!(detect("\"\\z\\\"\"€", 0), 6); // non-ascii after "\z\""
        assert_eq!(detect("\"€\"", 0), 5); // three-byte non-ascii in ""
        assert_eq!(detect("\"a€\"", 0), 6); // non-ascii in "a"
        assert_eq!(detect("\"\\€\"", 0), 6); // non-ascii in "\"
        assert_eq!(detect("\"\\z€\"", 0), 7); // non-ascii in "\z"
        assert_eq!(detect("\"\\z\\€\"", 0), 8); // non-ascii in "\z\"
        assert_eq!(detect("r\"€", 0), 0); // non-ascii after r"
        assert_eq!(detect("r\"a€", 0), 0); // non-ascii after r"a
        assert_eq!(detect("r\"\\€", 0), 0); // non-ascii after r"\
        assert_eq!(detect("r\"\\z€", 0), 0); // non-ascii after r"\z
        assert_eq!(detect("r\"\\z\\€", 0), 0); // non-ascii after r"\z\
        assert_eq!(detect("r\"\\z\\\"€", 0), 0); // non-ascii after r"\z\"
        assert_eq!(detect("r\"\\z\\\"\"€", 0), 7); // non-ascii after r"\z\""
        assert_eq!(detect("r\"€\"", 0), 6); // non-ascii in r""
        assert_eq!(detect("r\"a€\"", 0), 7); // non-ascii in r"a"
        assert_eq!(detect("r\"\\€\"", 0), 7); // non-ascii in r"\"
        assert_eq!(detect("r\"\\z€\"", 0), 8); // non-ascii in r"\z"
        assert_eq!(detect("r\"\\z\\€\"", 0), 9); // non-ascii in r"\z\"
        assert_eq!(detect("r#\"€", 0), 0); // non-ascii after r#"
        assert_eq!(detect("r#\"a€", 0), 0); // non-ascii after r#"a
        assert_eq!(detect("r#\"\\€", 0), 0); // non-ascii after r#"\
        assert_eq!(detect("r#\"\\z€", 0), 0); // non-ascii after r#"\z
        assert_eq!(detect("r#\"\\z\\€", 0), 0); // non-ascii after r#"\z\
        assert_eq!(detect("r#\"\\z\\\"€", 0), 0); // non-ascii after r#"\z\"
        assert_eq!(detect("r#\"\\z\"€", 0), 0); // non-ascii after r#"\z"
        assert_eq!(detect("r#\"€\"", 0), 0); // non-ascii in r#""
        assert_eq!(detect("r#\"a€\"", 0), 0); // non-ascii in r#"a"
        assert_eq!(detect("r#\"\\€\"", 0), 0); // non-ascii in r#"\"
        assert_eq!(detect("r#\"\\z€\"", 0), 0); // non-ascii in r#"\z"
        assert_eq!(detect("r#\"\\z\\€\"", 0), 0); // non-ascii in r#"\z\"
        assert_eq!(detect("r#\"\\z\\€\\\"\"#", 0), 13); // r#"\z\€\""#
        assert_eq!(detect("r##\"\\z\\€\\\"\"#", 0), 0); // missing hash at end
    }

}