use std::{fmt::Display, str::FromStr};

use num::Integer;
use once_cell::sync::Lazy;
use regex::Regex;

pub static NUMBER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<prefix>0[xb])?(?P<number>[\dabcdef_]+)").unwrap());
pub static WORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[_\w--[\d]]+").unwrap());

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Token {
    Word(String),
    Number(u16),
    LongNumber(u32),
    SpecialSymbol(char),
    EndOfInput,
}

impl Token {
    /// Returns `true` if the token is [`SpecialSymbol`].
    ///
    /// [`SpecialSymbol`]: Token::SpecialSymbol
    #[must_use]
    pub fn is_special_symbol(&self, expected: char) -> bool {
        matches!(self, &Self::SpecialSymbol(actual) if actual == expected)
    }
}

#[derive(Default)]
pub struct TokenStream {
    tokens: Vec<Token>,
    cursor: usize,
}

#[derive(Debug)]
pub enum TokenStreamError {
    UnexpectedToken(Token),
}

impl Display for TokenStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenStreamError::UnexpectedToken(token) => writeln!(f, "Unexpected token {token:?}"),
        }
    }
}

impl FromStr for TokenStream {
    type Err = TokenStreamError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut token_stream = TokenStream::default();
        let mut index = 0;

        // regex are utf8 aware by default
        // so we can safely assume, that their len() + current_index
        // always lies at a valid unicode char boundary
        // Therefore accessing char from byte offset into string becomes trivial:
        // string[offset..].chars().next().unwrap()

        // index is byte offset therefore len usage is valid
        while index < s.len() {
            // string subslicing along with `^` anchor is the official way
            // to do match *exactly at*. This approach is advertised by developers
            // as correct one and trivial to implement. As such, they won't 
            // add this functionality into library's api.
            
            // `find_at` changes starting position without stripping the string.
            // Hence, regex that has ^ anchor never match when used with `find_at`.
            // Regex without ^ anchor skips any symbols. These skipped symbols
            // may include those, which we define as invalid or 
            // which require special treatment e.g. ":"
            if let Some(found) = WORD_REGEX.find(&s[index..]) {
                token_stream
                    .tokens
                    .push(Token::Word(found.as_str().to_owned()));

                assert!(found.len() > 0, "match should be NON-empty");
                index += found.len();
                continue;
            }

            if let Some((number, length)) = parse_number(&s[index..]) {
                token_stream.tokens.push(Token::Number(number));

                index += length;
                continue;
            }

            // long numbers
            if let Some((number, length)) = parse_number(&s[index..]) {
                token_stream.tokens.push(Token::LongNumber(number));

                index += length;
                continue;
            }

            let symbol = s[index..].chars().next().expect("index points to valid unicode boundary and at least one char is available");
            if symbol == ' ' {
                index += symbol.len_utf8();
                continue;
            }

            token_stream.tokens.push(Token::SpecialSymbol(symbol));
            index += symbol.len_utf8();
        }

        token_stream.tokens.push(Token::EndOfInput);

        Ok(token_stream)
    }
}

impl TokenStream {
    pub fn peek(&self, n: usize) -> Result<&Token, TokenStreamError> {
        self.tokens
            .get(self.cursor + n - 1)
            .ok_or(TokenStreamError::UnexpectedToken(Token::EndOfInput))
    }

    fn advance_cursor(&mut self) {
        self.cursor += 1;
    }

    pub fn next_word(&mut self) -> Result<String, TokenStreamError> {
        let output = match self.peek(1) {
            Ok(Token::Word(word)) => word,
            Ok(token) => return Err(TokenStreamError::UnexpectedToken(token.clone())),
            Err(err) => return Err(err),
        }.clone();

        self.advance_cursor();

        Ok(output)
    }

    pub fn next_number(&mut self) -> Result<u16, TokenStreamError> {
        let output = match self.peek(1) {
            Ok(&Token::Number(number)) => number,
            Ok(token) => return Err(TokenStreamError::UnexpectedToken(token.clone())),
            Err(err) => return Err(err),
        };

        self.advance_cursor();

        Ok(output)
    }

    pub fn next_long_number(&mut self) -> Result<u32, TokenStreamError> {
        let output = match self.peek(1) {
            Ok(&Token::LongNumber(number)) => number,
            Ok(&Token::Number(number)) => number as u32,
            Ok(token) => return Err(TokenStreamError::UnexpectedToken(token.clone())),
            Err(err) => return Err(err),
        };

        self.advance_cursor();

        Ok(output)
    }

    pub fn next_special_symbol(&mut self, expected: char) -> Result<char, TokenStreamError> {
        let output = match self.peek(1) {
            Ok(&Token::SpecialSymbol(actual)) if actual == expected => actual,
            Ok(token) => return Err(TokenStreamError::UnexpectedToken(token.clone())),
            Err(err) => return Err(err),
        };

        self.advance_cursor();

        Ok(output)
    }

    pub fn next_end_of_input(&mut self) -> Result<(), TokenStreamError> {
        let output = match self.peek(1) {
            Ok(Token::EndOfInput) => (),
            Ok(token) => return Err(TokenStreamError::UnexpectedToken(token.clone())),
            Err(err) => return Err(err),
        };

        self.advance_cursor();

        Ok(output)
    }
}

// default number parser does not provide useful information anyway
// conversion of associated `FromStrRadix` type is hard or even impossible
// due to conflicting implementation of <T> From<T> for T
fn parse_number<T: Integer>(input: &str) -> Option<(T, usize)> {
    let parsed = NUMBER_REGEX
        .captures(input)?;

    let prefix = parsed.name("prefix").map_or("", |matched| matched.as_str());
    let value = &parsed["number"];

    let value = value.replace("_", "");
    let radix = match prefix {
        "0x" => 16,
        "0b" => 2,
        _ => 10,
    };

    if let Err(err) = T::from_str_radix(&value, radix) {
        let _a = err;
    }

    Some((
        T::from_str_radix(&value, radix).ok()?,
        parsed
            .get(0)
            .expect("`Some` is guaranteed by regex lib.")
            .len(),
    ))
}
