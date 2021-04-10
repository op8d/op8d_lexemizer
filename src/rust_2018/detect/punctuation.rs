//! Detects sequences of Punctuation characters, like `;` or `>>=`.

use super::super::lexeme::LexemeKind;
const DETECTED: LexemeKind = LexemeKind::Punctuation;
const UNDETECTED: (LexemeKind, usize) = (LexemeKind::Undetected, 0);

/// Detects sequences of Punctuation characters, like `;` or `>>=`.
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `chr` The character position in `orig` to look at
/// 
/// ### Returns
/// If `chr` begins a valid looking sequence of Punctuation characters,
/// `detect_punctuation()` returns `LexemeKind::Punctuation` and the character
/// position after it ends.  
/// Otherwise, `detect_punctuation()` returns `LexemeKind::Undetected` and `0`.
pub fn detect_punctuation(
    orig: &str,
    chr: usize,
) -> (
    LexemeKind,
    usize,
) {
    // If the current char is past the last char in `orig`, bail out!
    let len = orig.len();
    if chr >= len { return UNDETECTED }
    // If the current char is not present in PUNCTUATION_1, it is not, and does
    // not begin, punctuation. That’s because PUNCTUATION_2 and PUNCTUATION_3
    // all start with a PUNCTUATION_1 character.
    let c0 = orig.get(chr..chr+1).unwrap_or("~");
    if ! PUNCTUATION_1.contains(&c0) { return UNDETECTED }

    // If the current char is the last in the code, then it must be punctuation.
    if len == chr + 1 { return (DETECTED, len) }

    // Get two chars. If they are not a 2-char punctuation, then detect just
    // the single-character punctuation.
    let c1 = orig.get(chr..chr+2).unwrap_or("~");
    if ! PUNCTUATION_2.contains(&c1) { return (DETECTED, chr + 1) }

    // If c1 reaches the end of the code, then c0 starts a 2-char punctuation.
    if len == chr + 2 { return (DETECTED, len) }

    // Get three chars. If they are not a 3-char punctuation, then detect just
    // the two-character punctuation.
    let c2 = orig.get(chr..chr+3).unwrap_or("~");
    if ! PUNCTUATION_3.contains(&c2) { return (DETECTED, chr + 2) }

    // `detect_punctuation()` accepts any character at all after finding
    // 3-char punctuation. It could also be the end-of-input.
    (DETECTED, chr + 3)
}

const PUNCTUATION_1: [&str; 28] = [
    "'", // SingleQuote        Labels, Lifetimes
    "_", // Underscore         Wildcard patterns, Inferred types, Unnamed...
    "-", // Minus              Subtraction, Negation
    ",", // Comma              Various separators
    ";", // Semi               Terminator for situations, Array types
    ":", // Colon              Various separators
    "!", // Not                Bitwise and Logical NOT, Macro Calls, ...
    "?", // Question           Question mark operator, Questionably sized, ...
    ".", // Dot                Field access, Tuple index
    "(", // OpenParentheses    Logic
    ")", // CloseParentheses   Logic
    "[", // OpenSquareBraces   Arrays
    "]", // CloseSquareBraces  Arrays
    "{", // OpenCurlyBraces    Blocks
    "}", // CloseCurlyBraces   Blocks
    "@", // At                 Subpattern binding
    "*", // Star               Multiplication, Dereference, Raw Pointers, ...
    "/", // Slash              Division
    "&", // And                Bitwise / Logical AND, Borrow, References, ...
    "#", // Pound              Attributes
    "%", // Percent            Remainder
    "^", // Caret              Bitwise and Logical XOR
    "+", // Plus               Addition, Trait Bounds, Macro Kleene Matcher
    "<", // Lt                 Less than, Generics, Paths
    "=", // Eq                 Assignment, Attributes, Various type definitions
    ">", // Gt                 Greater than, Generics, Paths
    "|", // Or                 Bitwise / Logical OR, Closures, if let, ...
    "$", // Dollar             Macros
];

const PUNCTUATION_2: [&str; 20] = [
    "-=", // MinusEq        Subtraction assignment
    "->", // RArrow         Function return type, Closure return type, ...
    "::", // PathSep        Path separator
    "!=", // Ne             Not Equal
    "..", // DotDot         Range, Struct expressions, Patterns
    "*=", // StarEq         Multiplication assignment
    "/=", // SlashEq        Division assignment
    "&&", // AndAnd         Lazy AND, Borrow, References, Reference patterns
    "&=", // AndEq          Bitwise And assignment
    "%=", // PercentEq      Remainder assignment
    "^=", // CaretEq        Bitwise XOR assignment
    "+=", // PlusEq         Addition assignment
    "<<", // Shl            Shift Left, Nested Generics
    "<=", // Le             Less than or equal to
    "==", // EqEq           Equal
    "=>", // FatArrow       Match arms, Macros
    ">=", // Ge             Greater than or equal to, Generics
    ">>", // Shr            Shift Right, Nested Generics
    "|=", // OrEq           Bitwise Or assignment
    "||", // OrOr           Lazy OR, Closures
];

const PUNCTUATION_3: [&str; 4] = [
    "...", // DotDotDot  Variadic functions, Range patterns
    "..=", // DotDotEq   Inclusive Range, Range patterns
    "<<=", // ShlEq      Shift Left assignment
    ">>=", // ShrEq      Shift Right assignment, Nested Generics
];


#[cfg(test)]
mod tests {
    use super::detect_punctuation as detect;
    use super::DETECTED as D;
    use super::UNDETECTED as U;

    #[test]
    fn detect_punctuation_correct() {
        // Basic.
        let orig = "- === 'label ...";
        assert_eq!(detect(orig, 0),  (D,1)); // -
        assert_eq!(detect(orig, 2),  (D,4)); // == there is no "===" in Rust
        assert_eq!(detect(orig, 3),  (D,5)); // == finds the 2nd and 3rd char in ===
        assert_eq!(detect(orig, 6),  (D,7)); // ' not considered part of the label
        assert_eq!(detect(orig, 13), (D,16)); // ...

        // Single at end.
        assert_eq!(detect(" '", 1), (D,2));
        assert_eq!(detect(" _", 1), (D,2));
        assert_eq!(detect(" -", 1), (D,2));
        assert_eq!(detect(" ,", 1), (D,2));
        assert_eq!(detect(" ;", 1), (D,2));
        assert_eq!(detect(" :", 1), (D,2));
        assert_eq!(detect(" !", 1), (D,2));
        assert_eq!(detect(" ?", 1), (D,2));
        assert_eq!(detect(" .", 1), (D,2));
        assert_eq!(detect(" (", 1), (D,2));
        assert_eq!(detect(" )", 1), (D,2));
        assert_eq!(detect(" [", 1), (D,2));
        assert_eq!(detect(" ]", 1), (D,2));
        assert_eq!(detect(" {", 1), (D,2));
        assert_eq!(detect(" }", 1), (D,2));
        assert_eq!(detect(" @", 1), (D,2));
        assert_eq!(detect(" *", 1), (D,2));
        assert_eq!(detect(" /", 1), (D,2));
        assert_eq!(detect(" &", 1), (D,2));
        assert_eq!(detect(" #", 1), (D,2));
        assert_eq!(detect(" %", 1), (D,2));
        assert_eq!(detect(" ^", 1), (D,2));
        assert_eq!(detect(" +", 1), (D,2));
        assert_eq!(detect(" <", 1), (D,2));
        assert_eq!(detect(" =", 1), (D,2));
        assert_eq!(detect(" >", 1), (D,2));
        assert_eq!(detect(" |", 1), (D,2));
        assert_eq!(detect(" $", 1), (D,2));
        // Single then tilde.
        assert_eq!(detect(" '~", 1), (D,2));
        assert_eq!(detect(" _~", 1), (D,2));
        assert_eq!(detect(" -~", 1), (D,2));
        assert_eq!(detect(" ,~", 1), (D,2));
        assert_eq!(detect(" ;~", 1), (D,2));
        assert_eq!(detect(" :~", 1), (D,2));
        assert_eq!(detect(" !~", 1), (D,2));
        assert_eq!(detect(" ?~", 1), (D,2));
        assert_eq!(detect(" .~", 1), (D,2));
        assert_eq!(detect(" (~", 1), (D,2));
        assert_eq!(detect(" )~", 1), (D,2));
        assert_eq!(detect(" [~", 1), (D,2));
        assert_eq!(detect(" ]~", 1), (D,2));
        assert_eq!(detect(" {~", 1), (D,2));
        assert_eq!(detect(" }~", 1), (D,2));
        assert_eq!(detect(" @~", 1), (D,2));
        assert_eq!(detect(" *~", 1), (D,2));
        assert_eq!(detect(" /~", 1), (D,2));
        assert_eq!(detect(" &~", 1), (D,2));
        assert_eq!(detect(" #~", 1), (D,2));
        assert_eq!(detect(" %~", 1), (D,2));
        assert_eq!(detect(" ^~", 1), (D,2));
        assert_eq!(detect(" +~", 1), (D,2));
        assert_eq!(detect(" <~", 1), (D,2));
        assert_eq!(detect(" =~", 1), (D,2));
        assert_eq!(detect(" >~", 1), (D,2));
        assert_eq!(detect(" |~", 1), (D,2));
        assert_eq!(detect(" $~", 1), (D,2));
        // Single then equals.
        // Subset of single-char punctuation which should be terminated by "=".
        assert_eq!(detect(" '=", 1), (D,2));
        assert_eq!(detect(" _=", 1), (D,2));
        assert_eq!(detect(" ,=", 1), (D,2));
        assert_eq!(detect(" ;=", 1), (D,2));
        assert_eq!(detect(" :=", 1), (D,2));
        assert_eq!(detect(" ?=", 1), (D,2));
        assert_eq!(detect(" .=", 1), (D,2));
        assert_eq!(detect(" (=", 1), (D,2));
        assert_eq!(detect(" )=", 1), (D,2));
        assert_eq!(detect(" [=", 1), (D,2));
        assert_eq!(detect(" ]=", 1), (D,2));
        assert_eq!(detect(" {=", 1), (D,2));
        assert_eq!(detect(" }=", 1), (D,2));
        assert_eq!(detect(" @=", 1), (D,2));
        assert_eq!(detect(" #=", 1), (D,2));
        assert_eq!(detect(" $=", 1), (D,2));

        // Double at end.
        assert_eq!(detect(" -=", 1), (D,3));
        assert_eq!(detect(" ->", 1), (D,3));
        assert_eq!(detect(" ::", 1), (D,3));
        assert_eq!(detect(" !=", 1), (D,3));
        assert_eq!(detect(" ..", 1), (D,3));
        assert_eq!(detect(" *=", 1), (D,3));
        assert_eq!(detect(" /=", 1), (D,3));
        assert_eq!(detect(" &&", 1), (D,3));
        assert_eq!(detect(" &=", 1), (D,3));
        assert_eq!(detect(" %=", 1), (D,3));
        assert_eq!(detect(" ^=", 1), (D,3));
        assert_eq!(detect(" +=", 1), (D,3));
        assert_eq!(detect(" <<", 1), (D,3));
        assert_eq!(detect(" <=", 1), (D,3));
        assert_eq!(detect(" ==", 1), (D,3));
        assert_eq!(detect(" =>", 1), (D,3));
        assert_eq!(detect(" >=", 1), (D,3));
        assert_eq!(detect(" >>", 1), (D,3));
        assert_eq!(detect(" |=", 1), (D,3));
        assert_eq!(detect(" ||", 1), (D,3));
        // Double then tilde.
        assert_eq!(detect(" -=~", 1), (D,3));
        assert_eq!(detect(" ->~", 1), (D,3));
        assert_eq!(detect(" ::~", 1), (D,3));
        assert_eq!(detect(" !=~", 1), (D,3));
        assert_eq!(detect(" ..~", 1), (D,3));
        assert_eq!(detect(" *=~", 1), (D,3));
        assert_eq!(detect(" /=~", 1), (D,3));
        assert_eq!(detect(" &&~", 1), (D,3));
        assert_eq!(detect(" &=~", 1), (D,3));
        assert_eq!(detect(" %=~", 1), (D,3));
        assert_eq!(detect(" ^=~", 1), (D,3));
        assert_eq!(detect(" +=~", 1), (D,3));
        assert_eq!(detect(" <<~", 1), (D,3));
        assert_eq!(detect(" <=~", 1), (D,3));
        assert_eq!(detect(" ==~", 1), (D,3));
        assert_eq!(detect(" =>~", 1), (D,3));
        assert_eq!(detect(" >=~", 1), (D,3));
        assert_eq!(detect(" >>~", 1), (D,3));
        assert_eq!(detect(" |=~", 1), (D,3));
        assert_eq!(detect(" ||~", 1), (D,3));
        // Double then equals.
        // Subset of double-char punctuation which should be terminated by "=".
        assert_eq!(detect(" -==", 1), (D,3));
        assert_eq!(detect(" ->=", 1), (D,3));
        assert_eq!(detect(" ::=", 1), (D,3));
        assert_eq!(detect(" !==", 1), (D,3));
        assert_eq!(detect(" *==", 1), (D,3));
        assert_eq!(detect(" /==", 1), (D,3));
        assert_eq!(detect(" &&=", 1), (D,3));
        assert_eq!(detect(" &==", 1), (D,3));
        assert_eq!(detect(" %==", 1), (D,3));
        assert_eq!(detect(" ^==", 1), (D,3));
        assert_eq!(detect(" +==", 1), (D,3));
        assert_eq!(detect(" <==", 1), (D,3));
        assert_eq!(detect(" ===", 1), (D,3));
        assert_eq!(detect(" =>=", 1), (D,3));
        assert_eq!(detect(" >==", 1), (D,3));
        assert_eq!(detect(" |==", 1), (D,3));
        assert_eq!(detect(" ||=", 1), (D,3));

        // Triple at end.
        assert_eq!(detect(" ...", 1), (D,4));
        assert_eq!(detect(" ..=", 1), (D,4));
        assert_eq!(detect(" <<=", 1), (D,4));
        assert_eq!(detect(" >>=", 1), (D,4));
        // Triple then tilde.
        assert_eq!(detect(" ...~", 1), (D,4));
        assert_eq!(detect(" ..=~", 1), (D,4));
        assert_eq!(detect(" <<=~", 1), (D,4));
        assert_eq!(detect(" >>=~", 1), (D,4));
        // Triple then equals.
        // All triple-char punctuation should be terminated by "=".
        assert_eq!(detect(" ...=", 1), (D,4));
        assert_eq!(detect(" ..==", 1), (D,4));
        assert_eq!(detect(" <<==", 1), (D,4));
        assert_eq!(detect(" >>==", 1), (D,4));
    }

    #[test]
    fn detect_punctuation_incorrect() {
        let orig = "` =* .:.";
        assert_eq!(detect(orig, 0),  U);     // backtick is not Rust punctuation
        assert_eq!(detect(orig, 2), (D, 3)); // the = of =* is accepted
        assert_eq!(detect(orig, 5), (D, 6)); // the . of .:. is accepted
    }

    #[test]
    fn detect_punctuation_will_not_panic() {
        // Near the end of `orig`.
        assert_eq!(detect("",   0),  U);     // empty string
        assert_eq!(detect("~",  0),  U);     // tilde is not Rust punctuation
        assert_eq!(detect(">",  0), (D, 1)); // >
        // Invalid `chr`.
        assert_eq!(detect("abc", 2),   U); // 2 is before "c", so in range
        assert_eq!(detect("abc", 3),   U); // 3 is after "c", so incorrect
        assert_eq!(detect("abc", 4),   U); // 4 is out of range
        assert_eq!(detect("abc", 100), U); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1),     U); // part way into the three € bytes
        assert_eq!(detect(".€", 0),   (D,1)); // non-ascii after .
        assert_eq!(detect("..€", 0),  (D,2)); // non-ascii after ..
        assert_eq!(detect("...€", 0), (D,3)); // non-ascii after ...
    }

}
