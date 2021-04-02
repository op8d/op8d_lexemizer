//! Detects a multiline or inline comment.

/// Detects a multiline or inline comment.
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `pos` The character position in `orig` to look at
/// 
/// ### Returns
/// If `pos` begins a valid looking comment, `detect_comment()` returns
/// the character position after the comment ends.  
/// Otherwise, `detect_comment()` just returns the `pos` argument.
pub fn detect_comment(orig: &str, pos: usize) -> usize {
    // If the current char is the last or second-from-last in `orig`, it does not
    // begin a comment.
    let len = orig.len();
    if len < pos + 2 { return pos }
    // If the current char is not a forward slash, it does not begin a comment.
    if get_aot(orig, pos) != "/" { return pos }
    // If the next char is:
    match get_aot(orig, pos+1) {
        // Also a forward slash, `pos` could begin an inline comment.
        "/" => detect_inline_comment(orig, pos, len),
        // An asterisk, `pos` could begin a multiline comment.
        "*" => detect_multiline_comment(orig, pos, len),
        // Anything else, `pos` does not begin a comment.
        _ => pos,
    }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, p: usize) -> &str { orig.get(p..p+1).unwrap_or("~") }

fn detect_inline_comment(orig: &str, pos: usize, len: usize) -> usize {
    // Step through each char, from `pos + 2` to the end of the input code.
    let mut i = pos + 2;
    while i < len - 1 {
        // Get this character, even if it’s non-ascii.
        let mut j = i + 1;
        while !orig.is_char_boundary(j) { j += 1 }
        // If this char is a newline:
        if &orig[i..j] == "\n" { //@TODO maybe recognise Windows style "\r\n"?
            // Advance to the start of the newline.
            return i
        }
        // Step forward, ready for the next iteration.
        i = j;
    }
    // No newline was found, so advance to the end of the input code.
    len
}

fn detect_multiline_comment(orig: &str, pos: usize, len: usize) -> usize {
    // Track how deep into a nested multiline comment we are.
    let mut depth = 0;
    // Slightly hacky way to to skip forward while looping.
    let mut i = pos + 2;
    // Step through each char, from `pos` to the end of the original input code.
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
                return i + 2
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
    pos
}


#[cfg(test)]
mod tests {
    use super::detect_comment as detect;

    #[test]
    fn detect_comment_inline() {
        // With newline.
        let orig = "abc//ok\nxyz";
        assert_eq!(detect(orig, 2), 2); // c//o
        assert_eq!(detect(orig, 3), 7); // //ok advance four places
        assert_eq!(detect(orig, 4), 4); // /ok<NL>
        // Without newline.
        let orig = "abc//okxyz";
        assert_eq!(detect(orig, 2), 2);  // c//o
        assert_eq!(detect(orig, 3), 10); // //okxyz advance to the end
        assert_eq!(detect(orig, 4), 4);  // /okxyz
        // With Windows line ending. The carriage return, '\r ', is treated like
        // any other character.
        let orig = "abc//ok\r\nxyz";
        assert_eq!(detect(orig, 2), 2); // c//ok
        assert_eq!(detect(orig, 3), 8); // //ok<CR> advance five places
        assert_eq!(detect(orig, 4), 4); // /ok<CR><NL>
        // Non-ascii.
        assert_eq!(detect("//€", 0), 5); // 3-byte non-ascii directly after //
        assert_eq!(detect("//abcd€", 0), 9); // 3-byte non-ascii after //abcd
    }

    #[test]
    fn detect_comment_multiline_basic() {
        // Contains newline.
        let orig = "abc/*ok\n*/z";
        assert_eq!(detect(orig, 2), 2);  // c/*ok<NL>*
        assert_eq!(detect(orig, 3), 10); // /*ok<NL>*/ adv. seven places
        assert_eq!(detect(orig, 4), 4);  // *ok<NL>*/z
        // Doc.
        assert_eq!(detect("/** Here's a doc */", 0), 19);
        assert_eq!(detect("/**A/*A*/*/", 0), 11);
        assert_eq!(detect("/**A/*A'*/*/", 0), 12);
        // To end of `orig`.
        let orig = "abc/*ok*/";
        assert_eq!(detect(orig, 2), 2); // c/*ok*/
        assert_eq!(detect(orig, 3), 9); // /*ok*/ advance to the end
        assert_eq!(detect(orig, 4), 4); // *ok*/
        // Minimal.
        let orig = "//";
        assert_eq!(detect(orig, 0), 2);  // //
        assert_eq!(detect(orig, 1), 1);  // /
        let orig = "//\n";
        assert_eq!(detect(orig, 0), 3);  // //<NL>
        assert_eq!(detect(orig, 1), 1);  // /<NL>
        let orig = "/**/";
        assert_eq!(detect(orig, 0), 4);  // /**/
        assert_eq!(detect(orig, 1), 1);  // **/
        // Without end.
        let orig = "abc/*nope*";
        assert_eq!(detect(orig, 2), 2); // c/*nope*
        assert_eq!(detect(orig, 3), 3); // /*nope* malformed
        assert_eq!(detect(orig, 4), 4); // *nope*
    }
  
    #[test]
    fn detect_comment_multiline_nested() {
        // Single nesting.
        let orig = "/* outer /* inner */ outer */";
        assert_eq!(detect(orig, 0), 29); // does not end after ...inner */
        assert_eq!(detect(orig, 9), 20); // just catched /* inner */
        // Complex nesting.
        let orig = "pre-/* 0 /* 1 */ 0 /* 2 /* 3 */ 2 */ 0 */-post";
        assert_eq!(detect(orig, 3), 3);  // -/* 0
        assert_eq!(detect(orig, 4), 41); // /* 0 ... 0 */
        assert_eq!(detect(orig, 5), 5);  // * 0
        assert_eq!(detect(orig, 9), 16); // /* 1 */
        assert_eq!(detect(orig, 19), 36); // /* 2 /* 3 */ 2 */
        // `detect_comment()`’s loop deals with these edge cases correctly, by
        // stepping forward one extra pos after finding a nested "/*" or "*/".
        let orig = "/*/*/ */ */";
        assert_eq!(detect(orig, 0), 11); // /*/*/ */ */ edge case is the 3rd /
        assert_eq!(detect(orig, 1), 1);  // */*/ */ */
        assert_eq!(detect(orig, 2), 8);  // /*/ */
        let orig = "/*/* */* */";
        assert_eq!(detect(orig, 0), 11); // /*/* */* */ edge case is the 4th *
        assert_eq!(detect(orig, 1), 1);  // */* */* */
        assert_eq!(detect(orig, 2), 7);  // /* */
        // Invalid nesting.
        let orig = "/* outer /* inner */ missing trailing slash *";
        assert_eq!(detect(orig, 0), 0);
    }

    #[test]
    fn detect_comment_will_not_panic() {
        // Near the end of `orig`.
        assert_eq!(detect("", 0), 0); // empty string
        assert_eq!(detect("/", 0), 0); // /
        assert_eq!(detect("xyz/", 3), 3); // /
        assert_eq!(detect("*", 0), 0); // *
        assert_eq!(detect("//", 0), 2); // //
        assert_eq!(detect("//\n", 0), 3); // //<NL>
        assert_eq!(detect("//abc", 0), 5); // //abc
        assert_eq!(detect("//abc\n", 0), 6); // //abc<NL>
        assert_eq!(detect("/*", 0), 0); // /*
        assert_eq!(detect("*/", 0), 0); // */
        assert_eq!(detect("/**/", 0), 4); // /**/
        assert_eq!(detect("/*abc", 0), 0); // /*abc
        assert_eq!(detect("/*abc*", 0), 0); // /*abc*
        assert_eq!(detect("/*abc*/", 0), 7); // /*abc*/
        assert_eq!(detect("/*abc*/\n", 0), 7); // /*abc*/<NL>
        assert_eq!(detect("/*abc\n*/", 0), 8); // /*abc<NL>*/
        // Invalid `pos`.
        assert_eq!(detect("abc", 2), 2); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3), 3); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4), 4); // 4 is out of range
        assert_eq!(detect("abc", 100), 100); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1), 1); // part way through the three eurobytes
        assert_eq!(detect("/€", 0), 0); // non-ascii after /
        assert_eq!(detect("/*€", 0), 0); // non-ascii after /*
    }
  
}
