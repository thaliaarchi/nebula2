// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use bstr::ByteSlice;

use crate::ws::syntax::IntLiteral;

pub enum Token {
    Word,
    Int(IntLiteral),
    Char(String),
    String(String),
    LF,
    Comment(Vec<u8>),
}

pub struct Lexer<'a> {
    src: &'a [u8],
    offset: usize,
    prev_lf: bool,
}

pub enum TokenError {
    UnterminatedComment,
    InvalidUtf8,
}

impl<'a> Lexer<'a> {
    #[inline]
    pub const fn new<B>(src: &'a B) -> Self
    where
        B: ~const AsRef<[u8]> + ?Sized,
    {
        Lexer {
            src: src.as_ref(),
            offset: 0,
            prev_lf: false,
        }
    }

    #[inline]
    fn string(&mut self) -> Result<Token, TokenError> {
        todo!()
    }

    #[inline]
    fn char(&mut self) -> Result<Token, TokenError> {
        todo!()
    }

    #[inline]
    fn line_comment(&mut self, tag_len: usize) -> Result<Token, TokenError> {
        let lf = self.src[self.offset + tag_len..]
            .find_byte(b'\n')
            .map_or(self.src.len(), |i| i + 1);
        let comment = &self.src[self.offset..lf];
        self.offset = lf;
        Ok(Token::Comment(comment.to_owned()))
    }

    #[inline]
    fn block_comment(&mut self, end1: u8, end2: u8) -> Result<Token, TokenError> {
        let offset = self.offset;
        self.offset += 3;
        loop {
            match self.src[self.offset..].find_byte(end2) {
                Some(i) => {
                    self.offset = i + 1;
                    if self.src[i - 1] == end1 {
                        return Ok(Token::Comment(self.src[offset..self.offset].to_owned()));
                    }
                }
                None => {
                    self.offset = self.src.len();
                    return Err(TokenError::UnterminatedComment);
                }
            }
        }
    }

    #[inline]
    fn block_comment_nested(&mut self, start: u8, mid: u8, end: u8) -> Result<Token, TokenError> {
        let offset = self.offset;
        self.offset += 3;
        let mut level = 1;
        loop {
            match self.src[self.offset..].find_byte(mid) {
                Some(i) => {
                    self.offset = i + 1;
                    if self.src[i - 1] == start {
                        level += 1;
                    } else if i + 1 != self.src.len() && self.src[i + 1] == end {
                        self.offset = i + 2;
                        level -= 1;
                        if level == 0 {
                            return Ok(Token::Comment(self.src[offset..self.offset].to_owned()));
                        }
                    }
                }
                None => {
                    self.offset = self.src.len();
                    return Err(TokenError::UnterminatedComment);
                }
            }
        }
    }
}

/*impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, TokenError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let offset = self.offset;
            let tok = match self.src[offset..] {
                [b'\n', ..] if !self.prev_lf => {
                    self.prev_lf = true;
                    self.offset += 1;
                    Ok(Token::LF)
                }
                [b'0'..=b'9' | b'+' | b'-', ..] => self.number_or_word(),
                [b'"', ..] => self.string(),
                [b'\'', ..] => self.char(),
                [b'#' | b';', ..] => self.line_comment(1),
                [b'/', b'/', ..] | [b'-', b'-', ..] => self.line_comment(2),
                [b'/', b'*', ..] => self.block_comment(b'*', b'/'),
                [b'{', b'-', ..] => self.block_comment_nested(b'{', b'-', b'}'),
                [b'(', b'*', ..] => self.block_comment_nested(b'(', b'*', b')'),
                [ch, ..] if ch.is_ascii_whitespace() => {
                    self.offset += 1;
                    continue;
                }
                [ch, ..] if ch > 0x7f => {
                    let (ch, size) = bstr::decode_utf8(&self.src[offset..]);
                    self.offset += size;
                    match ch {
                        Some(ch) if ch.is_whitespace() => continue,
                        None => Err(TokenError::InvalidUtf8),
                    }
                }
                [ch, ..] => {}
            };
            return Some(tok);
        }
    }
}*/
