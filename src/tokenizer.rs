use crate::{
    buffer::Buffer,
    token::*,
    syntax::*,
    Result,
};

pub struct Tokenizer {
    buffer: Buffer<char>,
    row: u16,
    col: u16,
    digit_chars: Box<[char]>,
    ident_chars: Box<[char]>,
    op_chars: Box<[char]>,
}

pub struct TokenMeta {
    pub row: u16,
    pub col: u16,
}

impl Tokenizer {
    pub fn new(buffer: Buffer<char>) -> Self {
        fn to_chars(string: &'static str) -> Box<[char]> {
            string.chars().collect::<Vec<_>>().into_boxed_slice()
        }

        Self {
            buffer,
            row: 0,
            col: 0,
            digit_chars: to_chars(DIGITS),
            ident_chars: to_chars(IDENTIFIER_CHARS),
            op_chars: {
                let mut chars = Vec::new();
                for (_, string) in Operator::ARRAY.iter() {
                    for ch in string.chars() {
                        if !chars.contains(&ch) {
                            chars.push(ch);
                        }
                    }
                }
                chars.into_boxed_slice()
            },
        }
    }

    fn read_int(&mut self) -> Result<Token> {
        // TODO fix
        let mut string = String::new();
        for ch in self.buffer.next_from(&self.digit_chars) {
            string.push(ch);
        }
        Ok(Token::Integer(string.parse()?))
    }

    fn read_str(&mut self, term: char) -> Result<Token> {
        self.buffer.step();
        let mut string = String::new();
        loop {
            let ch = self.buffer.next()?.clone();
            if ch == term {
                break
            } else {
                string.push(ch);
            }
        }
        Ok(Token::String(string))
    }

    fn read_word(&mut self) -> Result<Token> {
        let word: String = self.buffer.next_from(&self.ident_chars).iter().collect();
        if let Some(kw) = Keyword::try_from_str(&word) {
            return Ok(Token::Keyword(kw.clone()))
        }
        if !IDENTIFIER_OPENERS.contains(word.chars().nth(0).unwrap()) {
            Err("invalid identifier".into())
        } else {
            Ok(Token::Identifier(word))
        }
    }

    fn read_operator(&mut self) -> Result<Token> {
        let string = &self.buffer.peek_from(&self.op_chars).iter().collect::<String>();
        let op = Operator::try_from_str(string);
        match op {
            Some(op) => {
                self.buffer.stepn(string.len());
                Ok(Token::Operator(op))
            }
            None => {
                Ok(Token::Operator(
                    Operator::try_from_str(&String::from(*self.buffer.next()?))
                    .ok_or(format!("invalid operator: {string}"))?
                ))
            }
        }
    }

    fn read_token(&mut self, ch: char) -> Result<Token> {
        if DIGITS.contains(ch) {
            self.read_int()
        } else if STR_TERMINATORS.contains(ch) {
            self.read_str(ch)
        } else if IDENTIFIER_OPENERS.contains(ch) {
            self.read_word()
        } else if self.op_chars.contains(&ch) {
            self.read_operator()
        } else {
            return Err(format!("unexpected character: {ch}").into())
        }
    }

    pub fn tokenize(&mut self) -> Result<(Vec<Token>, Vec<TokenMeta>)> {
        let mut tokens = Vec::new();
        let mut metas = Vec::new();

        loop {
            if let Ok(ch) = self.buffer.peek().cloned() {
                let meta = TokenMeta {
                    row: self.row,
                    col: self.col,
                };

                if !ch.is_ascii_whitespace() {
                    let i = self.buffer.i as u16;
                    let token = self.read_token(ch)?;
                    tokens.push(token);
                    metas.push(meta);
                    self.col += self.buffer.i as u16 - i;

                } else {
                    self.buffer.step();
                    if ch == '\n' {
                        self.row += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }
                }

            } else {
                break
            }
        }

        assert_eq!(tokens.len(), metas.len());

        Ok((tokens, metas))
    }
}
