//! An enum and a struct used by `lexemize()`.

use std::fmt;

///
/// ```txt
/// 0000000000000000000000000000XXXX   0 -  3  Character
/// 000000000000000000000000XXXX0000   4 -  7  Comment
/// 00000000000000000000XXXX00000000   8 - 11  Identifier
/// 0000000000000000XXXX000000000000  12 - 15  Number
/// 000000000000XXXX0000000000000000  16 - 19  Punctuation
/// 00000000XXXX00000000000000000000  20 - 23  String
/// 0000XXXX000000000000000000000000  24 - 27  Undetected, etc
/// XXXX0000000000000000000000000000  28 - 31  Whitespace
/// ```
/// 
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum LexemeKind {
    /// Not used yet.
    CharacterByte = 1,
    /// 
    CharacterHex = 2,
    /// 
    CharacterPlain = 4,
    /// 
    CharacterUnicode = 8,

    /// Not used yet.
    CommentDocInline = 16,
    /// Not used yet.
    CommentDocMultiline = 32,
    /// 
    CommentInline = 64,
    /// 
    CommentMultiline = 128,

    /// 
    IdentifierFreeword = 256,
    /// 
    IdentifierKeyword = 512,
    /// Not used yet.
    IdentifierOther = 1024,
    /// 
    IdentifierStdType = 2048,

    /// 
    NumberBinary = 4096,
    /// 
    NumberHex = 8192,
    /// 
    NumberOctal = 16384,
    /// 
    NumberDecimal = 32768,

    /// 
    Punctuation = 65536,

    /// Not used yet.
    StringByte = 1048576,
    /// Not used yet.
    StringByteRaw = 2097152,
    /// 
    StringPlain = 4194304,
    /// 
    StringRaw = 8388608,

    ///
    Undetected = 16777216,
    /// 
    Unexpected = 33554432,
    /// 
    Unidentifiable = 67108864,

    /// 
    WhitespaceTrimmable = 268435456,
}

///
#[derive(Copy, Clone)]
pub struct Lexeme {
    /// Category of the Lexeme.
    pub kind: LexemeKind,
    /// The position that the Lexeme starts, relative to the start of `orig`.
    /// Zero indexed.
    pub chr: usize,
    /// 
    pub snippet: &'static str,
}

impl fmt::Display for Lexeme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let kind = format!("{:?}", self.kind);
        let snippet = self.snippet.replace("\n", "<NL>");
        write!(fmt, "{: <20} {: >4}  {}", kind, self.chr, snippet)
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
    fn lexeme_kind_debug_as_expected() {
        assert_eq!(format!("{:?}", LexemeKind::CharacterByte),
                                              "CharacterByte");
        assert_eq!(format!("{:?}", LexemeKind::CharacterHex),
                                              "CharacterHex");
        assert_eq!(format!("{:?}", LexemeKind::CharacterPlain),
                                              "CharacterPlain");
        assert_eq!(format!("{:?}", LexemeKind::CharacterUnicode),
                                              "CharacterUnicode");
        assert_eq!(format!("{:?}", LexemeKind::CommentDocInline),
                                              "CommentDocInline");
        assert_eq!(format!("{:?}", LexemeKind::CommentDocMultiline),
                                              "CommentDocMultiline");
        assert_eq!(format!("{:?}", LexemeKind::CommentInline),
                                              "CommentInline");
        assert_eq!(format!("{:?}", LexemeKind::CommentMultiline),
                                              "CommentMultiline");
        assert_eq!(format!("{:?}", LexemeKind::IdentifierFreeword),
                                              "IdentifierFreeword");
        assert_eq!(format!("{:?}", LexemeKind::IdentifierKeyword),
                                              "IdentifierKeyword");
        assert_eq!(format!("{:?}", LexemeKind::IdentifierOther),
                                              "IdentifierOther");
        assert_eq!(format!("{:?}", LexemeKind::IdentifierStdType),
                                              "IdentifierStdType");
        assert_eq!(format!("{:?}", LexemeKind::NumberBinary),
                                              "NumberBinary");
        assert_eq!(format!("{:?}", LexemeKind::NumberHex),
                                              "NumberHex");
        assert_eq!(format!("{:?}", LexemeKind::NumberOctal),
                                              "NumberOctal");
        assert_eq!(format!("{:?}", LexemeKind::NumberDecimal),
                                              "NumberDecimal");
        assert_eq!(format!("{:?}", LexemeKind::Punctuation),
                                              "Punctuation");
        assert_eq!(format!("{:?}", LexemeKind::StringByte),
                                              "StringByte");
        assert_eq!(format!("{:?}", LexemeKind::StringByteRaw),
                                              "StringByteRaw");
        assert_eq!(format!("{:?}", LexemeKind::StringPlain),
                                              "StringPlain");
        assert_eq!(format!("{:?}", LexemeKind::StringRaw),
                                              "StringRaw");
        assert_eq!(format!("{:?}", LexemeKind::Undetected),
                                              "Undetected");
        assert_eq!(format!("{:?}", LexemeKind::Unexpected),
                                              "Unexpected");
        assert_eq!(format!("{:?}", LexemeKind::Unidentifiable),
                                              "Unidentifiable");
        assert_eq!(format!("{:?}", LexemeKind::WhitespaceTrimmable),
                                              "WhitespaceTrimmable");
    }

    #[test]
    fn lexeme_to_string_as_expected() {
        let lexeme = Lexeme {
            kind: LexemeKind::CharacterUnicode,
            chr: 123,
            snippet: "yup".into(),
        };
        assert_eq!(lexeme.to_string(), "CharacterUnicode      123  yup");
    }
}
