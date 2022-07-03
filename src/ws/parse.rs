// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use crate::ws::inst::Opcode;
use crate::ws::token::Token;

#[derive(Clone, Debug)]
pub struct Parser {
    entries: Vec<ParseEntry>,
}

#[derive(Clone, Debug, Default)]
enum ParseEntry {
    #[default]
    None,
    Prefix(Vec<Opcode>),
    Terminal(Opcode),
}

impl Parser {
    pub fn with_len(len: usize) -> Self {
        let mut entries = Vec::new();
        entries.resize(len, ParseEntry::None);
        Parser { entries }
    }

    pub fn register(&mut self, _toks: &[Token], _opcode: Opcode) {
        todo!();
    }
}
