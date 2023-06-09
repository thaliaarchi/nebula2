// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! Whitespace language.
//!
//! # Resources
//!
//! - [Whitespace tutorial](https://web.archive.org/web/20150618184706/http://compsoc.dur.ac.uk/whitespace/tutorial.php)
//! - [Reference interpreter](https://web.archive.org/web/20150717140342/http://compsoc.dur.ac.uk/whitespace/download.php)
//! - [The Whitespace Corpus](https://github.com/wspace/corpus)
//! - [Esolang wiki](https://esolangs.org/wiki/Whitespace)

pub use token::Token;

pub mod assembly;
pub mod gmh;
pub mod inst;
pub mod parse;
pub mod syntax;
pub mod token;

#[cfg(test)]
mod tests;
