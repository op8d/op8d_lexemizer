//! Detects a Freeword like `foo`, Keyword like `if` or StdType like `i8`.

use super::super::lexeme::LexemeKind;
const FREEWORD: LexemeKind = LexemeKind::IdentifierFreeword;
const KEYWORD: LexemeKind = LexemeKind::IdentifierKeyword;
const STD_TYPE: LexemeKind = LexemeKind::IdentifierStdType;
const UNDETECTED: (LexemeKind, usize) = (LexemeKind::Undetected, 0);

/// Detects a Freeword like `foo`, Keyword like `if` or StdType like `i8`.
/// 
/// ‘Freeword’ is what we’re calling any identifier which is not a Keyword or
/// StdType. For example the variable `i` or function name `get_widgets`.
///
/// Because of the way it’s used, `String` is categorised as a Freeword: @TODO maybe revisit this
/// `let s = String::from("hello");`
///
/// @TODO raw Identifiers, which have the `r#` prefix
///
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `chr` The character position in `orig` to look at
///
/// ### Returns
/// If `chr` begins a valid looking Identifier, `detect_identifier()` returns
/// its `LexemeKind` and the character position after the Identifier ends.  
/// Otherwise, `detect_identifier()` returns `LexemeKind::Undetected` and `0`.
pub fn detect_identifier(
    orig: &str,
    chr: usize,
) -> (
    LexemeKind,
    usize,
) {
    // If the current char is past the last char in `orig`, bail out!
    let len = orig.len();
    if chr >= len { return UNDETECTED }

    // If the current char is not [_a-zA-Z], it does not begin an Identifier.
    let c0 = get_aot(orig, chr);
    let c0_u = c0 == "_"; // true if the current char is an underscore
    if ! c0_u && ! c0.chars().all(char::is_alphabetic) { return UNDETECTED }
    // If the current char is the last in the input code:
    if len == chr + 1 {
        // A lone "_" is not an Identifier, but anything ascii-alphabetic is.
        // It can’t be a Keyword or StdType — they need 2 or more chars.
        return if c0_u { UNDETECTED } else { (FREEWORD, len) }
    }

    // Get the next character (or if it’s non-ascii, get a tilde).
    // If it’s not an underscore, letter or digit:
    let c1 = orig.get(chr+1..chr+2).unwrap_or("~");
    if c1 != "_" && ! c1.chars().all(char::is_alphanumeric) {
        // A lone "_" is not an Identifier, but anything ascii-alphabetic is.
        // It can’t be a Keyword or StdType — they need 2 or more chars.
        return if c0_u { UNDETECTED } else { (FREEWORD, chr + 1) }
    }

    // Step through each char, from two places after `chr` to the end of input.
    for i in chr+2..len {
        let c = get_aot(orig, i);
        // If this char is not an underscore, letter or digit, we detected
        // a Freeword, Keyword or StdType.
        if c != "_" && ! c.chars().all(char::is_alphanumeric) {
            return (categorize_identifier(&orig[chr..i]), i)
        }
    }
    // We reached the last char in the input code, so we detected a Freeword,
    // Keyword or StdType.
    (categorize_identifier(&orig[chr..len]), len)
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, c: usize) -> &str { orig.get(c..c+1).unwrap_or("~") }

fn categorize_identifier(s: &str) -> LexemeKind {
    // Look up the identifier in the `KEYWORDS` array.
    if KEYWORDS.contains(&s) { return KEYWORD }
    // Look up the identifier in the `STD_TYPE` array.
    if PRIMATIVE_TYPES.contains(&s) { return STD_TYPE }
    // Not recognised as a Keyword or StdType, so must be a Freeword.
    FREEWORD
}

const KEYWORDS: [&str; 52] = [
    "abstract",
    "as",
    "async",
    "await",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "final",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "pub",
    "ref",
    "return",
    "Self",
    "self",
    "static",
    // "'static" is a special case, detected during the refinement pass
    "struct",
    "super",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "union",
    "unsafe",
    "unsized",
    "use",
    "virtual",
    "where",
    "while",
    "yield",
];

const PRIMATIVE_TYPES: [&str; 18] = [
    "bool",
    "char",
    "f32",
    "f64",
    "i128",
    "i16",
    "i32",
    "i64",
    "i8",
    "isize",
    "str",
    "str",
    "u128",
    "u16",
    "u32",
    "u64",
    "u8",
    "usize",
];


#[cfg(test)]
mod tests {
    use super::detect_identifier as detect;
    use super::FREEWORD as F;
    use super::KEYWORD as K;
    use super::STD_TYPE as S;
    use super::UNDETECTED as U;

    #[test]
    fn detect_identifier_correct() {
        // Basic.
        let orig = "let^_def,G_h__1_; _123e+__ X2 Y Z foo!";
        assert_eq!(detect(orig, 0),  (K, 3)); // let
        assert_eq!(detect(orig, 1),  (F, 3)); // et
        assert_eq!(detect(orig, 2),  (F, 3)); // t
        assert_eq!(detect(orig, 3),   U);     // ^
        assert_eq!(detect(orig, 4),  (F, 8)); // _def
        assert_eq!(detect(orig, 8),   U);     // , is invalid in Identifiers
        assert_eq!(detect(orig, 9),  (F,16)); // G_h__1_
        assert_eq!(detect(orig, 18), (F,23)); // _123e
        assert_eq!(detect(orig, 24), (F,26)); // __
        assert_eq!(detect(orig, 27), (F,29)); // X2
        assert_eq!(detect(orig, 30), (F,31)); // Y
        assert_eq!(detect(orig, 32), (F,33)); // Z
        // `foo` not `foo!`, because macros are detected during refinement.
        assert_eq!(detect(orig, 34), (F,37)); // foo

        // Keywords basic.
        let orig = "as break const";
        assert_eq!(detect(orig, 0), (K,2));  // if
        assert_eq!(detect(orig, 3), (K,8));  // then
        assert_eq!(detect(orig, 9), (K,14)); // else

        // Keywords exhaustive.
        // doc.rust-lang.org/reference/keywords.html
        assert_eq!(detect("as",       0), (K,2));
        assert_eq!(detect("do",       0), (K,2));
        assert_eq!(detect("fn",       0), (K,2));
        assert_eq!(detect("if",       0), (K,2));
        assert_eq!(detect("in",       0), (K,2));
        assert_eq!(detect("box",      0), (K,3));
        assert_eq!(detect("dyn",      0), (K,3));
        assert_eq!(detect("for",      0), (K,3));
        assert_eq!(detect("let",      0), (K,3));
        assert_eq!(detect("mod",      0), (K,3));
        assert_eq!(detect("mut",      0), (K,3));
        assert_eq!(detect("pub",      0), (K,3));
        assert_eq!(detect("ref",      0), (K,3));
        assert_eq!(detect("try",      0), (K,3));
        assert_eq!(detect("use",      0), (K,3));
        assert_eq!(detect("else",     0), (K,4));
        assert_eq!(detect("enum",     0), (K,4));
        assert_eq!(detect("impl",     0), (K,4));
        assert_eq!(detect("loop",     0), (K,4));
        assert_eq!(detect("move",     0), (K,4));
        assert_eq!(detect("priv",     0), (K,4));
        assert_eq!(detect("Self",     0), (K,4));
        assert_eq!(detect("self",     0), (K,4));
        assert_eq!(detect("true",     0), (K,4));
        assert_eq!(detect("type",     0), (K,4));
        assert_eq!(detect("await",    0), (K,5));
        assert_eq!(detect("break",    0), (K,5));
        assert_eq!(detect("const",    0), (K,5));
        assert_eq!(detect("crate",    0), (K,5));
        assert_eq!(detect("false",    0), (K,5));
        assert_eq!(detect("final",    0), (K,5));
        assert_eq!(detect("macro",    0), (K,5));
        assert_eq!(detect("match",    0), (K,5));
        assert_eq!(detect("super",    0), (K,5));
        assert_eq!(detect("trait",    0), (K,5));
        assert_eq!(detect("union",    0), (K,5));
        assert_eq!(detect("where",    0), (K,5));
        assert_eq!(detect("while",    0), (K,5));
        assert_eq!(detect("yield",    0), (K,5));
        assert_eq!(detect("become",   0), (K,6));
        assert_eq!(detect("extern",   0), (K,6));
        assert_eq!(detect("return",   0), (K,6));
        assert_eq!(detect("static",   0), (K,6));
        assert_eq!(detect("struct",   0), (K,6));
        assert_eq!(detect("typeof",   0), (K,6));
        assert_eq!(detect("unsafe",   0), (K,6));
        assert_eq!(detect("unsized",  0), (K,7));
        assert_eq!(detect("virtual",  0), (K,7));
        assert_eq!(detect("abstract", 0), (K,8));
        assert_eq!(detect("continue", 0), (K,8));
        assert_eq!(detect("override", 0), (K,8));
        assert_eq!(detect("'static",  0),  U); // special case
        assert_eq!(detect("'static",  1), (K,7));

        // PrimativeTypes basic.
        let orig = "bool i128 isize";
        assert_eq!(detect(orig,  0), (S,4));  // bool
        assert_eq!(detect(orig,  5), (S,9));  // i128
        assert_eq!(detect(orig, 10), (S,15)); // isize

        // PrimativeTypes exhaustive.
        // doc.rust-lang.org/std/#primitives
        assert_eq!(detect("i8",    0), (S,2));
        assert_eq!(detect("u8",    0), (S,2));
        assert_eq!(detect("f32",   0), (S,3));
        assert_eq!(detect("f64",   0), (S,3));
        assert_eq!(detect("i16",   0), (S,3));
        assert_eq!(detect("i32",   0), (S,3));
        assert_eq!(detect("i64",   0), (S,3));
        assert_eq!(detect("str",   0), (S,3));
        assert_eq!(detect("str",   0), (S,3));
        assert_eq!(detect("u16",   0), (S,3));
        assert_eq!(detect("u32",   0), (S,3));
        assert_eq!(detect("u64",   0), (S,3));
        assert_eq!(detect("bool",  0), (S,4));
        assert_eq!(detect("char",  0), (S,4));
        assert_eq!(detect("i128",  0), (S,4));
        assert_eq!(detect("u128",  0), (S,4));
        assert_eq!(detect("isize", 0), (S,5));
        assert_eq!(detect("usize", 0), (S,5));
    }

    #[test]
    fn detect_identifier_incorrect() {
        // Here, each lone "_" exercises a different conditional branch.
        let orig = "_ 2X _";
        assert_eq!(detect(orig, 0), U); // _ cannot be the only char
        assert_eq!(detect(orig, 2), U); // 2X is not a valid Identifier
    }

    #[test]
    fn detect_identifier_will_not_panic() {
        // Near the end of `orig`.
        assert_eq!(detect("", 0),    U); // empty string
        assert_eq!(detect("'", 0),   U); // '
        assert_eq!(detect("'a", 0),  U); // 'a
        assert_eq!(detect("'a", 1), (F,2)); // a
        assert_eq!(detect("_", 0),   U); // _ cannot be the only char
        // Invalid `chr`.
        assert_eq!(detect("abc", 2),  (F,3)); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3),   U); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4),   U); // 4 is out of range
        assert_eq!(detect("abc", 100), U); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1),        U); // part way into the three € bytes
        assert_eq!(detect("a€", 0),      (F,1)); // a
        assert_eq!(detect("abcd€fg", 2), (F,4)); // cd
    }
}
