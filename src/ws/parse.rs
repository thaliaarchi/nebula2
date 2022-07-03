// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use crate::ws::inst::Opcode;
use crate::ws::token::{Token, TokenSeq};

#[derive(Clone, Debug)]
pub struct ParseTable {
    entries: Vec<ParseEntry>,
}

#[derive(Clone, Debug, Default)]
enum ParseEntry {
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
    pub fn with_len(len: usize) -> Self {
        let mut entries = Vec::new();
        entries.resize(len, ParseEntry::None);
        ParseTable { entries }
    }

    pub fn insert(&mut self, toks: &[Token], opcode: Opcode) -> Result<(), ParserError> {
        if toks.len() == 0 {
            return Err(ParserError::EmptyTokenSeq);
        }
        let mut seq = TokenSeq::new();
        for &tok in toks {
            let entry = &mut self.entries[seq.0 as usize];
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
        let entry = &mut self.entries[seq.0 as usize];
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
