//! Transforms Rust 2018 code to a vector of Lexemes.

use std::fmt::{Display,Formatter,Error};

use super::lexeme::{Lexeme,LexemeKind};
use super::detect::character::detect_character;
use super::detect::comment::detect_comment;
use super::detect::identifier::detect_identifier;
use super::detect::number::detect_number;
use super::detect::punctuation::detect_punctuation;
use super::detect::string::detect_string;
use super::detect::whitespace::detect_whitespace;

///
pub struct LexemizeResult {
    ///
    pub lexemes: Vec<Lexeme>,
}

impl Display for LexemizeResult {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let mut out = format!("Lexemes, incl <EOI>: {}\n", self.lexemes.len());
        for lexeme in &self.lexemes {
            out.push_str(&lexeme.to_string());
            out.push_str("\n");
        }
        write!(fmt, "{}", out)
    }
}

/// An array which contains all the `detect_*()` functions, in the proper order.
/// 
/// We usually default to alphabetical order, but need to make one exception:
/// `String` can start with an `"r"` character, so `detect_string()` must be
/// placed before `detect_identifier()`.
pub const DETECTORS: [fn (&str, usize) -> (LexemeKind, usize); 7] = [
    detect_character,
    detect_comment,
    detect_string,
    detect_identifier,
    detect_number,
    detect_punctuation,
    detect_whitespace,
];

/// Transforms a Rust 2018 program into a vector of `Lexemes`.
/// 
/// The primary purpose of `lexemize()` is to quickly divide Rust code into
/// three basic sections — comments, strings, and everything else.
/// 
/// The ‘everything else’ section is then divided into literals, punctuation,
/// whitespace and identifiers. Anything left over is marked as ‘Unidentifiable’.
/// 
/// Any input string can be lexemized, so this function never returns any kind
/// of error. Checking `orig` for semantic correctness should be done later on,
/// when the context is known during parsing.
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// 
/// ### Returns
/// `lexemize()` returns a [`LexemizeResult`] object.
pub fn lexemize(
    orig: &'static str
) -> LexemizeResult {
    // Initialise `len`, and some mutable variables.
    let len = orig.len();
    let mut chr = 0;
    let mut unident_chr = 0;
    let mut lexemes: Vec<Lexeme> = vec![];

    // Loop until we reach the last character of the input.
    'outer: while chr < len {
        // Only try to detect a Lexeme if this is the start of a character.
        if orig.is_char_boundary(chr) {
            // Step through the array of `detect_*()` functions, and their
            // associated `LexemeKinds`.
            for detector in DETECTORS.iter() {

                // If `detector()` does not detect the Lexeme, it will return
                // the same char-position as `chr`. In that case, just return `chr`.
                let (kind, next_chr) = detector(orig, chr);
                if kind != LexemeKind::Undetected {

                    // If any ‘Unidentifiable’ characters precede this Lexeme,
                    // record them before recording this Lexeme.
                    if unident_chr != chr {
                        lexemes.push(Lexeme {
                            kind: LexemeKind::Unidentifiable,
                            chr: unident_chr,
                            snippet: &orig[unident_chr..chr],
                        });
                    }
                    lexemes.push(Lexeme {
                        kind,
                        chr,
                        snippet: &orig[chr..next_chr],
                    });

                    // Step forward to the position after this Lexeme.
                    chr = next_chr;
                    unident_chr = next_chr;
                    continue 'outer;
                }
            }
            // Anything else is an unidentifiable character, which will be
            // picked up by the `unident_chr != chr` conditional above.
        }

        // Step forward one byte.
        chr += 1;
    }

    // If there are unidentifiable characters at the end of `orig`, add a final 
    // `Unidentifiable` Lexeme before the end-of-input Lexeme.
    if unident_chr != chr {
        lexemes.push(Lexeme {
            kind: LexemeKind::Unidentifiable,
            chr: unident_chr,
            snippet: &orig[unident_chr..chr],
        });
    }

    // Add a special end-of-input Whitespace Lexeme. This simplifies parsing
    // code which does not already end in whitespace.
    lexemes.push(Lexeme {
        kind: LexemeKind::WhitespaceTrimmable,
        chr,
        snippet: "<EOI>",
    });

    // Create and return a result object.
    LexemizeResult {
        lexemes,
    }
}

fn _detect(
    detector: fn (&str, usize) -> usize,
    kind: LexemeKind,
    orig: &'static str,
    chr: usize,
    unident_chr: usize,
    lexemes: &mut Vec<Lexeme>,
) -> usize {
    // If the passed-in `detector()` does not detect the Lexeme, it will return
    // the same char-position as `chr`. In that case, just return `chr`.
    let next_chr = detector(orig, chr);
    if next_chr == chr { return chr }

    // If any ‘Unidentifiable’ characters precede this Lexeme, record them
    // before recording this Lexeme.
    if unident_chr != chr {
        lexemes.push(Lexeme {
            kind: LexemeKind::Unidentifiable,
            chr: unident_chr,
            snippet: &orig[unident_chr..chr],
        });
    }
    lexemes.push(Lexeme {
        kind,
        chr,
        snippet: &orig[chr..next_chr],
    });

    // Tell `lexemize()` the character position of the end of the Lexeme.
    next_chr
}


#[cfg(test)]
mod tests {
    use super::{LexemizeResult,lexemize};
    use super::super::lexeme::{Lexeme,LexemeKind};

    #[test]
    fn lexemize_result_to_string_as_expected() {
        let result = LexemizeResult {
            lexemes: vec![
                Lexeme {
                    kind: LexemeKind::CommentMultiline,
                    chr: 0,
                    snippet: "/* This is a comment */",
                },
                Lexeme {
                    kind: LexemeKind::NumberDecimal,
                    chr: 23,
                    snippet: "44.4",
                },
                Lexeme {
                    kind: LexemeKind::WhitespaceTrimmable,
                    chr: 27,
                    snippet: "<EOI>",
                },
            ],
        };
        assert_eq!(result.to_string(),
            "Lexemes, incl <EOI>: 3\n\
             CommentMultiline        0  /* This is a comment */\n\
             NumberDecimal          23  44.4\n\
             WhitespaceTrimmable    27  <EOI>\n"
        );
    }

    #[test]
    fn lexemize_all_lexemes() {
        // Empty string.
        assert_eq!(lexemize("").to_string(),
            "Lexemes, incl <EOI>: 1\n\
             WhitespaceTrimmable     0  <EOI>\n");
        // One of each basic Lexeme.
        assert_eq!(lexemize("'A'/*B*/C 1!\"D\"\n").to_string(),
            "Lexemes, incl <EOI>: 9\n\
             CharacterPlain          0  \'A\'\n\
             CommentMultiline        3  /*B*/\n\
             IdentifierFreeword      8  C\n\
             WhitespaceTrimmable     9   \n\
             NumberDecimal          10  1\n\
             Punctuation            11  !\n\
             StringPlain            12  \"D\"\n\
             WhitespaceTrimmable    15  <NL>\n\
             WhitespaceTrimmable    16  <EOI>\n");
        // One of each basic Lexeme, with non-ascii.
        assert_eq!(lexemize("'€'/*€*/€1!\"€\"\n").to_string(),
            "Lexemes, incl <EOI>: 8\n\
             CharacterPlain          0  \'€\'\n\
             CommentMultiline        5  /*€*/\n\
             Unidentifiable         12  €\n\
             NumberDecimal          15  1\n\
             Punctuation            16  !\n\
             StringPlain            17  \"€\"\n\
             WhitespaceTrimmable    22  <NL>\n\
             WhitespaceTrimmable    23  <EOI>\n");
        // A simple "Hello, World!" one-liner.
        assert_eq!(lexemize("println!(\"Hello, World!\");\n").to_string(),
            "Lexemes, incl <EOI>: 8\n\
             IdentifierFreeword      0  println\n\
             Punctuation             7  !\n\
             Punctuation             8  (\n\
             StringPlain             9  \"Hello, World!\"\n\
             Punctuation            24  )\n\
             Punctuation            25  ;\n\
             WhitespaceTrimmable    26  <NL>\n\
             WhitespaceTrimmable    27  <EOI>\n");
    }

    #[test]
    fn lexemize_characters() {
        // Three Characters.
        assert_eq!(lexemize("'Z''\\t''\\x3F''\\u{3F}'").to_string(),
            "Lexemes, incl <EOI>: 5\n\
             CharacterPlain          0  \'Z\'\n\
             CharacterPlain          3  \'\\t\'\n\
             CharacterHex            7  \'\\x3F\'\n\
             CharacterUnicode       13  \'\\u{3F}\'\n\
             WhitespaceTrimmable    21  <EOI>\n"
        );
    }

    #[test]
    fn lexemize_comments() {
        // Three Comments.
        assert_eq!(lexemize("/**A/*A'*/*///B\n//C").to_string(),
            "Lexemes, incl <EOI>: 5\n\
             CommentMultiline        0  /**A/*A'*/*/\n\
             CommentInline          12  //B\n\
             WhitespaceTrimmable    15  <NL>\n\
             CommentInline          16  //C\n\
             WhitespaceTrimmable    19  <EOI>\n"
        );
    }

    #[test]
    fn lexemize_identifiers() {
        // Three Identifiers.
        assert_eq!(lexemize("u32;_D,__12 as foo!").to_string(),
            "Lexemes, incl <EOI>: 11\n\
             IdentifierStdType       0  u32\n\
             Punctuation             3  ;\n\
             IdentifierFreeword      4  _D\n\
             Punctuation             6  ,\n\
             IdentifierFreeword      7  __12\n\
             WhitespaceTrimmable    11   \n\
             IdentifierKeyword      12  as\n\
             WhitespaceTrimmable    14   \n\
             IdentifierFreeword     15  foo\n\
             Punctuation            18  !\n\
             WhitespaceTrimmable    19  <EOI>\n"
        );
    }

    #[test]
    fn lexemize_numbers() {
        // Three Numbers.
        assert_eq!(lexemize("0b1001_0011 1_2.3_4E+_5_ 0x__01aB__ 0o1_7").to_string(),
            "Lexemes, incl <EOI>: 8\n\
             NumberBinary            0  0b1001_0011\n\
             WhitespaceTrimmable    11   \n\
             NumberDecimal          12  1_2.3_4E+_5_\n\
             WhitespaceTrimmable    24   \n\
             NumberHex              25  0x__01aB__\n\
             WhitespaceTrimmable    35   \n\
             NumberOctal            36  0o1_7\n\
             WhitespaceTrimmable    41  <EOI>\n"
        );
    }

    #[test]
    fn lexemize_punctuations() {
        // Three Punctuations.
        assert_eq!(lexemize(";*=>>=").to_string(),
            "Lexemes, incl <EOI>: 4\n\
             Punctuation             0  ;\n\
             Punctuation             1  *=\n\
             Punctuation             3  >>=\n\
             WhitespaceTrimmable     6  <EOI>\n"
        );
    }

    #[test]
    fn lexemize_strings() {
        // Three Strings.
        assert_eq!(lexemize("\"\"\"ok\"r##\"\\\"\"##").to_string(),
            "Lexemes, incl <EOI>: 4\n\
             StringPlain             0  \"\"\n\
             StringPlain             2  \"ok\"\n\
             StringRaw               6  r##\"\\\"\"##\n\
             WhitespaceTrimmable    15  <EOI>\n"
      );
    }

    #[test]
    fn lexemize_unidentifiable() {
        // Mixture.
        assert_eq!(lexemize("~¶ €").to_string(),
            "Lexemes, incl <EOI>: 4\n\
             Unidentifiable          0  ~¶\n\
             WhitespaceTrimmable     3   \n\
             Unidentifiable          4  €\n\
             WhitespaceTrimmable     7  <EOI>\n"
        );
        // Non-ascii.
        assert_eq!(lexemize("~`\\").to_string(),
            "Lexemes, incl <EOI>: 2\n\
             Unidentifiable          0  ~`\\\n\
             WhitespaceTrimmable     3  <EOI>\n"
        );
        // Ascii.
        assert_eq!(lexemize("é¢€±").to_string(),
            "Lexemes, incl <EOI>: 2\n\
             Unidentifiable          0  é¢€±\n\
             WhitespaceTrimmable     9  <EOI>\n"
        );
    }

    #[test]
    fn lexemize_whitespace() {
        // Three Whitespace.
        assert_eq!(lexemize("\t\ta \n\nb\r ").to_string(),
            "Lexemes, incl <EOI>: 6\n\
             WhitespaceTrimmable     0  \t\t\n\
             IdentifierFreeword      2  a\n\
             WhitespaceTrimmable     3   <NL><NL>\n\
             IdentifierFreeword      6  b\n\
             WhitespaceTrimmable     7  \r \n\
             WhitespaceTrimmable     9  <EOI>\n"
      );
    }
}
