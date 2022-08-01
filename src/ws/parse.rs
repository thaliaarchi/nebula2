// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::sync::LazyLock;

use strum::IntoEnumIterator;

use crate::syntax::{PrefixTable, TokenSeq};
use crate::token_vec;
use crate::ws::inst::Opcode;
use crate::ws::token::Token;

pub static TABLE: LazyLock<PrefixTable<Token, Opcode>> = LazyLock::new(|| {
    let dense_len = TokenSeq::from(token_vec![L L L]).as_usize() + 1;
    let mut table = PrefixTable::new(dense_len);
    for opcode in Opcode::iter() {
        let toks = Vec::from(opcode.tokens());
        table.insert(&toks, opcode).unwrap();
    }
    table
});
