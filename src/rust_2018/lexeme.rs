//! An enum and a struct used by `lexemize()`.

use std::fmt;

///
#[derive(Clone,Copy,PartialEq)]
pub enum LexemeKind {
    /// 
    Character,
    /// 
    Comment,
    /// 
    Identifier,
    /// 
    Number,
    /// 
    Punctuation,
    /// 
    String,
    /// 
    Unidentifiable,
    /// 
    Whitespace,
}

impl LexemeKind {
    /// @TODO impl fmt::Display for LexemeKind
    pub fn to_string(&self) -> &str {
        match self {
            Self::Character      => "Character",
            Self::Comment        => "Comment",
            Self::Identifier     => "Identifier",
            Self::Number         => "Number",
            Self::Punctuation    => "Punctuation",
            Self::String         => "String",
            Self::Unidentifiable => "Unidentifiable",
            Self::Whitespace     => "Whitespace",
        }
    }
}

///
pub struct Lexeme {
    /// Category of the Lexeme.
    pub kind: LexemeKind,
    /// The position that the Lexeme starts, relative to the start of `orig`.
    /// Zero indexed.
    pub pos: usize,
    /// 
    pub snippet: String,
}

impl fmt::Display for Lexeme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let kind = self.kind.to_string();
        let snippet = self.snippet.replace("\n", "<NL>");
        write!(fmt, "{: <16} {: >4}  {}", kind, self.pos, snippet)
        //                     |||
        //                     ||+-- target width is four characters
        //                     |+--- align right
        //                     +---- fill with spaces
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn lexeme_kind_to_string_as_expected() {
        assert_eq!(LexemeKind::Character.to_string(),      "Character");
        assert_eq!(LexemeKind::Comment.to_string(),        "Comment");
        assert_eq!(LexemeKind::Identifier.to_string(),     "Identifier");
        assert_eq!(LexemeKind::Number.to_string(),         "Number");
        assert_eq!(LexemeKind::Punctuation.to_string(),    "Punctuation");
        assert_eq!(LexemeKind::String.to_string(),         "String");
        assert_eq!(LexemeKind::Unidentifiable.to_string(), "Unidentifiable");
        assert_eq!(LexemeKind::Whitespace.to_string(),     "Whitespace");
    }

    #[test]
    fn lexeme_to_string_as_expected() {
        let lexeme = Lexeme {
            kind: LexemeKind::Character,
            pos: 123,
            snippet: "yup".into(),
        };
        assert_eq!(lexeme.to_string(), "Character         123  yup");
    }
}
