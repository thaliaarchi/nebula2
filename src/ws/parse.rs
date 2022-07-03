// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;

use crate::ws::inst::Opcode;
use crate::ws::token::{Token, Token::*, TokenSeq};

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

#[derive(Clone, Debug)]
pub enum ParserError {
    Conflict { seq: TokenSeq, opcodes: Vec<Opcode> },
    EmptyTokenSeq,
}

impl ParseTable {
    const DENSE_MAX: TokenSeq = TokenSeq::from_tokens(&[L, L, L]);
    const DENSE_LEN: usize = Self::DENSE_MAX.as_usize() + 1;

    pub fn new() -> Self {
        ParseTable {
            dense: Box::new([const { ParseEntry::None }; Self::DENSE_LEN]),
            sparse: HashMap::new(),
        }
    }

    pub fn get(&self, seq: TokenSeq) -> &ParseEntry {
        if seq <= Self::DENSE_MAX {
            &self.dense[seq.as_usize()]
        } else {
            &self.sparse[&seq]
        }
    }

    pub fn get_mut(&mut self, seq: TokenSeq) -> &mut ParseEntry {
        if seq <= Self::DENSE_MAX {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn insert(&mut self, toks: &[Token], opcode: Opcode) -> Result<(), ParserError> {
        if toks.len() == 0 {
            return Err(ParserError::EmptyTokenSeq);
        }
        let mut seq = TokenSeq::new();
        for &tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                ParseEntry::None => *entry = ParseEntry::Prefix(vec![opcode]),
                ParseEntry::Prefix(opcodes) => opcodes.push(opcode),
                ParseEntry::Terminal(terminal) => {
                    let opcodes = vec![*terminal, opcode];
                    return Err(ParserError::Conflict { seq, opcodes });
                }
            }
            seq = seq.push(tok);
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
