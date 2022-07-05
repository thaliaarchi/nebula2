// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;

use bitvec::prelude::BitVec;
use strum::IntoEnumIterator;

use crate::ws::inst::{Features, Inst, Int, Opcode, Sign, Uint};
use crate::ws::lex::{LexError, Lexer};
use crate::ws::token::{Token::*, TokenSeq};
use crate::ws::token_vec::token_vec;

#[derive(Clone, Debug)]
pub struct Parser {
    table: ParseTable,
    lex: Lexer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    LexError(LexError),
    UnknownInst(TokenSeq),
    IncompleteInst(Vec<Opcode>),
    SignlessInt(Opcode),
    UnterminatedArg(Opcode),
}

impl Parser {
    pub fn new(lex: Lexer, features: Features) -> Result<Self, ParserError> {
        let mut table = ParseTable::new();
        for opcode in Opcode::iter() {
            if opcode.feature().map_or(true, |f| features.contains(f)) {
                table.register(opcode)?;
            }
        }
        Ok(Parser { table, lex })
    }

    #[inline]
    fn parse_arg(&mut self, opcode: Opcode) -> Result<Inst, ParseError> {
        opcode.parse_arg(self)
    }

    pub(crate) fn parse_int(&mut self, opcode: Opcode) -> Result<Int, ParseError> {
        let sign = match self
            .lex
            .next()
            .transpose()?
            .ok_or(ParseError::UnterminatedArg(opcode))?
        {
            S => Sign::Pos,
            T => Sign::Neg,
            L => return Err(ParseError::SignlessInt(opcode)),
        };
        let bits = self.parse_bitvec(opcode)?;
        Ok(Int { sign, bits })
    }

    pub(crate) fn parse_uint(&mut self, opcode: Opcode) -> Result<Uint, ParseError> {
        let bits = self.parse_bitvec(opcode)?;
        Ok(Uint { bits })
    }

    fn parse_bitvec(&mut self, opcode: Opcode) -> Result<BitVec, ParseError> {
        let mut bits = BitVec::new();
        while let Some(tok) = self.lex.next().transpose()? {
            match tok {
                S => bits.push(false),
                T => bits.push(true),
                L => return Ok(bits),
            }
        }
        Err(ParseError::UnterminatedArg(opcode))
    }
}

impl Iterator for Parser {
    type Item = Result<Inst, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut seq = TokenSeq::new();
        let mut prefix = &Vec::new();
        loop {
            match self.lex.next() {
                Some(Ok(tok)) => seq.push(tok),
                Some(Err(err)) => return Some(Err(ParseError::LexError(err))),
                None if seq.empty() => return None,
                None => return Some(Err(ParseError::IncompleteInst(prefix.clone()))),
            }
            match self.table.get(seq) {
                ParseEntry::None => return Some(Err(ParseError::UnknownInst(seq))),
                ParseEntry::Prefix(opcodes) => prefix = opcodes,
                ParseEntry::Terminal(opcode) => return Some(self.parse_arg(*opcode)),
            }
        }
    }
}

impl const From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError::LexError(err)
    }
}

#[derive(Clone, Debug)]
pub struct ParseTable {
    dense: Box<[ParseEntry; Self::DENSE_LEN]>,
    sparse: HashMap<TokenSeq, ParseEntry>,
}

#[derive(Clone, Debug, Default)]
pub enum ParseEntry {
    #[default]
    None,
    Prefix(Vec<Opcode>),
    Terminal(Opcode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParserError {
    Conflict { seq: TokenSeq, opcodes: Vec<Opcode> },
    EmptyTokenSeq(Opcode),
}

impl ParseTable {
    const DENSE_MAX: TokenSeq = token_vec![L L L].into();
    const DENSE_LEN: usize = Self::DENSE_MAX.as_usize() + 1;

    pub fn new() -> Self {
        ParseTable {
            dense: vec![ParseEntry::None; Self::DENSE_LEN]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            sparse: HashMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, seq: TokenSeq) -> &ParseEntry {
        if seq <= Self::DENSE_MAX {
            &self.dense[seq.as_usize()]
        } else {
            &self.sparse[&seq]
        }
    }

    #[inline]
    pub fn get_mut(&mut self, seq: TokenSeq) -> &mut ParseEntry {
        if seq <= Self::DENSE_MAX {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn register(&mut self, opcode: Opcode) -> Result<(), ParserError> {
        let toks = opcode.tokens();
        if toks.len() == 0 {
            return Err(ParserError::EmptyTokenSeq(opcode));
        }
        let mut seq = TokenSeq::new();
        for tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                ParseEntry::None => *entry = ParseEntry::Prefix(vec![opcode]),
                ParseEntry::Prefix(opcodes) => opcodes.push(opcode),
                ParseEntry::Terminal(terminal) => {
                    let opcodes = vec![*terminal, opcode];
                    return Err(ParserError::Conflict { seq, opcodes });
                }
            }
            seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            ParseEntry::None => *entry = ParseEntry::Terminal(opcode),
            ParseEntry::Prefix(opcodes) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return Err(ParserError::Conflict { seq, opcodes });
            }
            ParseEntry::Terminal(terminal) => {
                let opcodes = vec![*terminal, opcode];
                return Err(ParserError::Conflict { seq, opcodes });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ws::test::{tutorial_insts, TUTORIAL_STL};
    use crate::ws::token::CharMapping;

    #[test]
    fn test_parse_tutorial() -> Result<(), ParseError> {
        let lex = Lexer::new(TUTORIAL_STL.to_owned().into_bytes(), CharMapping::STL);
        let parser = Parser::new(lex, Features::all()).unwrap();
        let insts = parser.collect::<Result<Vec<_>, ParseError>>()?;
        assert_eq!(tutorial_insts(), insts);
        Ok(())
    }
}
