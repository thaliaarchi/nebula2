// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

// Nightly features
#![feature(
    lazy_cell,
    let_chains,
    map_try_insert,
    never_type,
    split_array,
    trait_alias
)]
// Unstable features
#![feature(core_intrinsics)]
// Clippy lints
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::enum_glob_use,
    clippy::module_name_repetitions
)]

pub mod bf;
pub mod syntax;
pub mod text;
pub mod ws;
