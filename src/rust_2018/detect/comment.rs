//! Detects a multiline or inline comment.

use super::super::lexeme::LexemeKind;
const INLINE:  LexemeKind = LexemeKind::CommentInline;
const MULTILINE: LexemeKind = LexemeKind::CommentMultiline;
const UNDETECTED: (LexemeKind, usize) = (LexemeKind::Undetected, 0);


/// Detects a multiline or inline comment.
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `chr` The character position in `orig` to look at
/// 
/// ### Returns
/// If `chr` begins a valid looking comment, `detect_comment()` returns the
/// appropriate `LexemeKind::Comment*` and the position after the comment ends.  
/// Otherwise, `detect_comment()` returns `LexemeKind::Undetected` and `0`.
pub fn detect_comment(
    orig: &str,
    chr: usize,
) -> (
    LexemeKind,
    usize,
) {
    // If the current char is the last or second-from-last in `orig`, it does not
    // begin a comment.
    let len = orig.len();
    if len < chr + 2 { return UNDETECTED }
    // If the current char is not a forward slash, it does not begin a comment.
    if get_aot(orig, chr) != "/" { return UNDETECTED }
    // If the next char is:
    match get_aot(orig, chr+1) {
        // Also a forward slash, `chr` could begin an inline comment.
        "/" => detect_inline_comment(orig, chr, len),
        // An asterisk, `chr` could begin a multiline comment.
        "*" => detect_multiline_comment(orig, chr, len),
        // Anything else, `chr` does not begin a comment.
        _ => UNDETECTED,
    }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, c: usize) -> &str { orig.get(c..c+1).unwrap_or("~") }

fn detect_inline_comment(
    orig: &str,
    chr: usize,
    len: usize,
) -> (
    LexemeKind,
    usize,
) {
    // Step through each char, from `chr + 2` to the end of the input code.
    let mut i = chr + 2;
    while i < len - 1 {
        // Get this character, even if it’s non-ascii.
        let mut j = i + 1;
        while !orig.is_char_boundary(j) { j += 1 }
        // If this char is a newline:
        if &orig[i..j] == "\n" { //@TODO maybe recognise Windows style "\r\n"?
            // Advance to the start of the newline.
            return (INLINE, i)
        }
        // Step forward, ready for the next iteration.
        i = j;
    }
    // No newline was found, so advance to the end of the input code.
    (INLINE, len)
}

fn detect_multiline_comment(
    orig: &str,
    chr: usize,
    len: usize,
) -> (
    LexemeKind,
    usize,
) {
    // Track how deep into a nested multiline comment we are.
    let mut depth = 0;
    // Slightly hacky way to to skip forward while looping.
    let mut i = chr + 2;
    // Step through each char, from `chr` to the end of the original input code.
    while i < len {
        // Get this character, even if it’s non-ascii.
        let mut j = i + 1;
        while !orig.is_char_boundary(j) { j += 1 }
        let c0 = &orig[i..j];
        // Get the next character, or tilde if it’s non-ascii.
        let c1 = get_aot(orig, j);
        // If this char is an asterisk, and the next is a forward slash:
        if c0 == "*" && c1 == "/" {
            // If the depth is zero (so we are at the outermost nesting level):
            if depth == 0 {
                // Advance to the end of the "*/".
                return (MULTILINE, i + 2)
            // Otherwise we are some way inside a nested multiline comment:
            } else {
                // Decrement the nesting-depth.
                depth -= 1;
                // Skip the forward slash (avoids confusion in "/*/* */* */").
                j += 1;
            }
        // If this char is a forward slash, and the next is an asterisk:
        } else if c0 == "/" && c1 == "*" {
            // Increment the nesting-depth.
            depth += 1;
            // Skip the asterisk (avoids confusion in "/*/*/ */ */").
            j += 1;
        }
        // Step forward, ready for the next iteration.
        i = j;
    }
    // The outermost "*/" was not found, so this is not a multiline comment.
    UNDETECTED
}


#[cfg(test)]
mod tests {
    use super::detect_comment as detect;
    use super::INLINE as I;
    use super::MULTILINE as M;
    use super::UNDETECTED as U;

    #[test]
    fn detect_comment_inline() {
        // With newline.
        let orig = "abc//ok\nxyz";
        assert_eq!(detect(orig, 2),  U);    // c//o
        assert_eq!(detect(orig, 3), (I,7)); // //ok advance four places
        assert_eq!(detect(orig, 4),  U);    // /ok<NL>
        // Without newline.
        let orig = "abc//okxyz";
        assert_eq!(detect(orig, 2),  U);     // c//o
        assert_eq!(detect(orig, 3), (I,10)); // //okxyz advance to the end
        assert_eq!(detect(orig, 4),  U);     // /okxyz
        // With Windows line ending. The carriage return, '\r ', is treated like
        // any other character.
        let orig = "abc//ok\r\nxyz";
        assert_eq!(detect(orig, 2),  U);    // c//ok
        assert_eq!(detect(orig, 3), (I,8)); // //ok<CR> advance five places
        assert_eq!(detect(orig, 4),  U);    // /ok<CR><NL>
        // Minimal.
        let orig = "//";
        assert_eq!(detect(orig, 0), (I,2)); // //
        assert_eq!(detect(orig, 1),  U);    // /
        let orig = "//\n";
        assert_eq!(detect(orig, 0), (I,3)); // //<NL>
        assert_eq!(detect(orig, 1),  U);    // /<NL>
        // Non-ascii.
        assert_eq!(detect("//€", 0),    (I,5)); // 3-byte non-ascii after //
        assert_eq!(detect("//abc€", 0), (I,8)); // 3-byte non-ascii after //abc
    }

    #[test]
    fn detect_comment_multiline_basic() {
        // Contains newline.
        let orig = "abc/*ok\n*/z";
        assert_eq!(detect(orig, 2),  U);     // c/*ok<NL>*
        assert_eq!(detect(orig, 3), (M,10)); // /*ok<NL>*/ adv. seven places
        assert_eq!(detect(orig, 4),  U);     // *ok<NL>*/z
        // Doc.
        assert_eq!(detect("/** Here's a doc */", 0), (M,19));
        assert_eq!(detect("/**A/*A*/*/", 0),         (M,11));
        assert_eq!(detect("/**A/*A'*/*/", 0),        (M,12));
        // To end of `orig`.
        let orig = "abc/*ok*/";
        assert_eq!(detect(orig, 2),  U);    // c/*ok*/
        assert_eq!(detect(orig, 3), (M,9)); // /*ok*/ advance to the end
        assert_eq!(detect(orig, 4),  U);    // *ok*/
        // Minimal.
        let orig = "/**/";
        assert_eq!(detect(orig, 0), (M,4)); // /**/
        assert_eq!(detect(orig, 1),  U);    // **/
        // Without end.
        let orig = "abc/*nope*";
        assert_eq!(detect(orig, 2),  U); // c/*nope*
        assert_eq!(detect(orig, 3),  U); // /*nope* malformed
        assert_eq!(detect(orig, 4),  U); // *nope*
    }
  
    #[test]
    fn detect_comment_multiline_nested() {
        // Single nesting.
        let orig = "/* outer /* inner */ outer */";
        assert_eq!(detect(orig, 0), (M,29)); // does not end after ...inner */
        assert_eq!(detect(orig, 9), (M,20)); // just catched /* inner */
        // Complex nesting.
        let orig = "pre-/* 0 /* 1 */ 0 /* 2 /* 3 */ 2 */ 0 */-post";
        assert_eq!(detect(orig, 3),  U);     // -/* 0
        assert_eq!(detect(orig, 4), (M,41)); // /* 0 ... 0 */
        assert_eq!(detect(orig, 5),  U);     // * 0
        assert_eq!(detect(orig, 9), (M,16)); // /* 1 */
        assert_eq!(detect(orig, 19),(M,36)); // /* 2 /* 3 */ 2 */
        // `detect_comment()`’s loop deals with these edge cases correctly, by
        // stepping forward one extra chr after finding a nested "/*" or "*/".
        let orig = "/*/*/ */ */";
        assert_eq!(detect(orig, 0), (M,11)); // /*/*/ */ */ edge case is 3rd /
        assert_eq!(detect(orig, 1),  U);     // */*/ */ */
        assert_eq!(detect(orig, 2), (M,8));  // /*/ */
        let orig = "/*/* */* */";
        assert_eq!(detect(orig, 0), (M,11)); // /*/* */* */ edge case is 4th *
        assert_eq!(detect(orig, 1),  U);     // */* */* */
        assert_eq!(detect(orig, 2), (M,7));  // /* */
        // Invalid nesting.
        let orig = "/* outer /* inner */ missing trailing slash *";
        assert_eq!(detect(orig, 0),  U);
    }

    #[test]
    fn detect_comment_will_not_panic() {
        // Near the end of `orig`.
        assert_eq!(detect("", 0), U); // empty string
        assert_eq!(detect("/", 0), U); // /
        assert_eq!(detect("xyz/", 3), U); // /
        assert_eq!(detect("*", 0), U); // *
        assert_eq!(detect("//", 0), (I,2)); // //
        assert_eq!(detect("//\n", 0), (I,3)); // //<NL>
        assert_eq!(detect("//abc", 0), (I,5)); // //abc
        assert_eq!(detect("//abc\n", 0), (I,6)); // //abc<NL>
        assert_eq!(detect("/*", 0), U); // /*
        assert_eq!(detect("*/", 0), U); // */
        assert_eq!(detect("/**/", 0), (M,4)); // /**/
        assert_eq!(detect("/*abc", 0), U); // /*abc
        assert_eq!(detect("/*abc*", 0), U); // /*abc*
        assert_eq!(detect("/*abc*/", 0), (M,7)); // /*abc*/
        assert_eq!(detect("/*abc*/\n", 0), (M,7)); // /*abc*/<NL>
        assert_eq!(detect("/*abc\n*/", 0), (M,8)); // /*abc<NL>*/
        // Invalid `chr`.
        assert_eq!(detect("abc", 2),   U); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3),   U); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4),   U); // 4 is out of range
        assert_eq!(detect("abc", 100), U); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1),     U); // part way into the three € bytes
        assert_eq!(detect("/€", 0),    U); // non-ascii after /
        assert_eq!(detect("/*€", 0),   U); // non-ascii after /*
    }
  
}
