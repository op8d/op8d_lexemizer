//! Detects an identifier, like `String` or `foo_bar`.

/// Detects an identifier, like `String` or `foo_bar`.
/// 
/// @TODO raw identifiers, which have the `r#` prefix
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `pos` The character position in `orig` to look at
/// 
/// ### Returns
/// If `pos` begins a valid looking identifier, `detect_identifier()`
/// returns the character position after the identifier ends.  
/// Otherwise, `detect_identifier()` just returns the `pos` argument.
pub fn detect_identifier(orig: &str, pos: usize) -> usize {
    // If the current char is past the last char in `orig`, bail out!
    let len = orig.len();
    if pos >= len { return pos }
    // If the current char is not [_a-zA-Z], it does not begin an identifier.
    let c = get_aot(orig, pos);
    let starts_u = c == "_"; // true if the current char is an underscore
    if ! starts_u && ! c.chars().all(char::is_alphabetic) { return pos }
    // If the current char is the last in the input code:
    if len == pos + 1 {
        // A lone "_" is not an identifier, but anything ascii-alphabetic is.
        return if starts_u { pos } else { len }
    }
    // If the next char is not an underscore, letter or digit:
    let c = orig.get(pos+1..pos+2).unwrap_or("/");
    if c != "_" && ! c.chars().all(char::is_alphanumeric) {
        // A lone "_" is not an identifier. Else, advance after the first char.
        return if starts_u { pos } else { pos + 1 }
    }
    // Step through each char, from `pos` to the end of the input code.
    for i in pos+2..len-1 {
        let c = get_aot(orig, i);
        // If this char is not an underscore, letter or digit, advance to here.
        if c != "_" && ! c.chars().all(char::is_alphanumeric) { return i }
    }
    // The last char in the input code is a valid identifier.
    len
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, p: usize) -> &str { orig.get(p..p+1).unwrap_or("~") }


#[cfg(test)]
mod tests {
    use super::detect_identifier as detect;
    
    #[test]
    fn detect_identifier_correct() {
        let orig = "abc^_def,G_h__1_; _123e+__ X2 Y Z";
        assert_eq!(detect(orig, 0), 3);   // abc
        assert_eq!(detect(orig, 1), 3);   // bc
        assert_eq!(detect(orig, 2), 3);   // c
        assert_eq!(detect(orig, 3), 3);   // ^ is invalid in identifiers
        assert_eq!(detect(orig, 4), 8);   // _def
        assert_eq!(detect(orig, 8), 8);   // , is invalid in identifiers
        assert_eq!(detect(orig, 9), 16);  // G_h__1_
        assert_eq!(detect(orig, 18), 23); // _123e
        assert_eq!(detect(orig, 24), 26); // __
        assert_eq!(detect(orig, 27), 29); // X2
        assert_eq!(detect(orig, 30), 31); // Y
        assert_eq!(detect(orig, 32), 33); // Z
    }

    #[test]
    fn detect_identifier_incorrect() {
        // Here, each lone "_" exercises a different conditional branch.
        let orig = "_ 2X _";
        assert_eq!(detect(orig, 0), 0); // _ cannot be the only char
        assert_eq!(detect(orig, 2), 2); // 2X is not a valid identifier
        assert_eq!(detect(orig, 5), 5); // _ cannot be the only char
    }

    #[test]
    fn detect_identifier_will_not_panic() {
        // Near the end of `orig`.
        assert_eq!(detect("", 0), 0); // empty string
        assert_eq!(detect("'", 0), 0); // '
        assert_eq!(detect("'a", 0), 0); // 'a
        assert_eq!(detect("'a", 1), 2); // a
        assert_eq!(detect("_", 0), 0); // _
        // Invalid `pos`.
        assert_eq!(detect("abc", 2), 3); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3), 3); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4), 4); // 4 is out of range
        assert_eq!(detect("abc", 100), 100); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1), 1); // part way through the three eurobytes
        assert_eq!(detect("a€", 0), 1); // a
        assert_eq!(detect("abcd€fg", 2), 4); // cd
    }

}
