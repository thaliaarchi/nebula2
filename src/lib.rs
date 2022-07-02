// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

#![feature(const_trait_impl, inline_const)]
#![allow(dead_code)]

pub mod ws {
    pub mod inst;
    pub mod token;

    pub use inst::{parse, Inst};
    pub use token::Token;
}
