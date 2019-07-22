extern crate logos;

use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
enum RawToken {
    // Logos requires that we define two default variants,
    // one for end of input source,
    #[end]
    End,

    // ...and one for errors. Those can be named anything
    // you wish as long as the attributes are there.
    #[error]
    Error,

    // Tokens can be literal strings, of any length.
    #[regex = "[[:digit:]]+"]
    Number,

    #[regex = "[[:alpha:]][[:alnum:]]*"]
    Command,

    #[token = ":"]
    Colon,

    // Or regular expressions.
    #[token = ";"]
    SemiColon,
}

/// Helper to convert RawTokens to Tokens
impl From<RawToken> for Token {
    fn from(t: RawToken) -> Self {
        match t {
            RawToken::End=>Token::End,
            RawToken::Error => 3,
            RawToken::PopOfEmptyStack => 4,
            RawToken::InvalidSyntax(_) => 5,
            RawToken::Io(_) => 6,
        }
    }
}