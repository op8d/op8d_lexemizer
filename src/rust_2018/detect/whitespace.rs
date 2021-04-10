//! Detects a sequence of Whitespace characters.

use super::super::lexeme::LexemeKind;
const DETECTED: LexemeKind = LexemeKind::WhitespaceTrimmable;
const UNDETECTED: (LexemeKind, usize) = (LexemeKind::Undetected, 0);

/// Detects a sequence of Whitespace characters.
/// 
/// Rust uses Pattern_White_Space, and treats it all the same.
/// There is some debate on whether to simplify things, in future:
/// internals.rust-lang.org/t/do-we-need-unicode-whitespace/9876
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `chr` The character position in `orig` to look at
/// 
/// ### Returns
/// If `chr` begins a sequence of Whitespace characters, `detect_whitespace()`
/// returns `LexemeKind::WhitespaceTrimmable` and the position after it ends.  
/// Otherwise, `detect_whitespace()` returns `LexemeKind::Undetected` and `0`.
pub fn detect_whitespace(
    orig: &str,
    chr: usize,
) -> (
    LexemeKind,
    usize,
) {
    // If the current char is past the last char in `orig`, or `chr` is not on
    // a character boundary, bail out! The char boundary test avoids a potential
    // panic if `&orig[i..j]` is reached, below.
    let len = orig.len();
    if chr >= len || !orig.is_char_boundary(chr) { return UNDETECTED }
    // Step through each byte-position, from `chr` to the end of the input code.
    let mut i = chr;
    while i < len {
        // Get the current character if it’s ascii, or get "~" if it’s not.
        let c = get_aot(orig, i);
        // Jump to the next char if this is ascii whitespace.
        if c == " "        // U+0020  UTF-8 20        "Space"
        || c == "\n"       // U+000A  UTF-8 0A        "New Line" or "Line Feed"
        || c == "\t"       // U+0009  UTF-8 09        "Horizontal Tabulation"
        || c == "\r"       // U+000D  UTF-8 0D        "Carriage Return"
        || c == "\u{000B}" // U+000B  UTF-8 0B        "Vertical Tabulation"
        || c == "\u{000C}" // U+000C  UTF-8 0C        "Form Feed"
        { i += 1; continue }
        // End the loop if this is ascii non-whitespace.
        if c != "~" { break }
        // End the loop if there is no next byte.
        if i >= len - 1 { break }
        // Get the next character.
        let mut j = i + 1;
        while !orig.is_char_boundary(j) { j += 1 }
        let c = &orig[i..j];
        // End the loop if we encountered a literal tilde.
        if c == "~" { break }
        // Jump to the next char if this is non-ascii Pattern_White_Space.
        if c == "\u{0085}" // U+0085  UTF-8 C2 85     "Next Line"
        || c == "\u{200E}" // U+200E  UTF-8 E2 80 8E  "Left-To-Right Mark"
        || c == "\u{200F}" // U+200F  UTF-8 E2 80 8F  "Right-To-Left Mark"
        || c == "\u{2028}" // U+2028  UTF-8 E2 80 A8  "Line Separator"
        || c == "\u{2029}" // U+2029  UTF-8 E2 80 A9  "Paragraph Separator"
        { i = j; continue }
        // End the loop if we encountered anything else.
        break;
    }
    // Advance to the end of the input code.
    if i == chr { UNDETECTED } else { (DETECTED, i) }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, c: usize) -> &str { orig.get(c..c+1).unwrap_or("~") }


#[cfg(test)]
mod tests {
    use super::detect_whitespace as detect;
    use super::DETECTED as D;
    use super::UNDETECTED as U;

    #[test]
    fn detect_whitespace_correct() {
        // Typical.
        let orig = "~abc \t\nxyz~";
        assert_eq!(detect(orig, 3),  U);    // c
        assert_eq!(detect(orig, 4), (D,7)); // <SP><TB><NL> advance three spaces
        assert_eq!(detect(orig, 5), (D,7)); // <TB><NL> advance two spaces
        assert_eq!(detect(orig, 6), (D,7)); // <NL> advance one space
        assert_eq!(detect(orig, 7),  U);    // xyz~

        // Exhaustive.
        // doc.rust-lang.org/reference/whitespace.html
        assert_eq!(detect("\u{0000}", 0),  U);    // null is not whitespace
        assert_eq!(detect("\u{0009}", 0), (D,1)); // horizontal tab
        assert_eq!(detect("\u{000A}", 0), (D,1)); // line feed
        assert_eq!(detect("\u{000B}", 0), (D,1)); // vertical tab
        assert_eq!(detect("\u{000C}", 0), (D,1)); // form feed
        assert_eq!(detect("\u{000D}", 0), (D,1)); // carriage return
        assert_eq!(detect("\u{0020}", 0), (D,1)); // space
        assert_eq!(detect("\u{0085}", 0), (D,2)); // next line
        assert_eq!(detect("\u{00A0}", 0),  U);    // NBSP is not whitespace
        assert_eq!(detect("\u{200E}", 0), (D,3)); // left-to-right
        assert_eq!(detect("\u{200F}", 0), (D,3)); // right-to-left
        assert_eq!(detect("\u{2028}", 0), (D,3)); // line separator
        assert_eq!(detect("\u{2029}", 0), (D,3)); // just paragraph separator
        let orig = "\u{0000}\u{0009}\u{000A}\u{000B}\u{000C}\u{000D}\u{0020}\u{0085}";
        assert_eq!(detect(orig, 0),  U);    // null is not whitespace
        assert_eq!(detect(orig, 1), (D,9)); // "next line" is two bytes
        let orig = "\u{00A0}\u{200E}\u{200F}\u{2028}\u{2029}";
        assert_eq!(detect(orig, 0),  U); // NBSP is not whitespace
        assert_eq!(detect(orig, 2), (D,14)); // 2 + (4 * 3)

        // Ends with newline.
        let orig = "xyz~ \n";
        assert_eq!(detect(orig, 2),  U);    // z~ <NL>
        assert_eq!(detect(orig, 3),  U);    // ~ <NL>
        assert_eq!(detect(orig, 4), (D,6)); //  <NL> advance to <EOI>
        assert_eq!(detect(orig, 5), (D,6)); // <NL> advance to <EOI>
    }

    #[test]
    fn detect_whitespace_will_not_panic() {
        // Near the end of `orig` input code.
        assert_eq!(detect("", 0),    U); // empty string
        assert_eq!(detect("~", 0),   U); // ~
        assert_eq!(detect("\n", 0), (D,1)); // <NL>
        // Invalid `chr`.
        assert_eq!(detect("abc", 2),   U); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3),   U); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4),   U); // 4 is out of range
        assert_eq!(detect("abc", 100), U); // 100 is way out of range
        let orig = "\u{00A0}\u{200E}\u{200F}\u{2028}\u{2029}";
        assert_eq!(detect(orig, 1), U); // `chr` halfway through NBSP
        // Non-ascii.
        assert_eq!(detect("€", 1),     U);    // part way into the three € bytes
        assert_eq!(detect(" €", 0),   (D,1)); // non-ascii after space
        assert_eq!(detect("\u{2029}€", 0), (D,3)); // non-ascii after U+2029
    }
}
