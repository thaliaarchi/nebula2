// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

#![feature(
    box_patterns,
    concat_bytes,
    const_array_into_iter_constructors,
    const_for,
    const_intoiterator_identity,
    const_mut_refs,
    const_option,
    const_option_ext,
    const_trait_impl,
    core_intrinsics,
    if_let_guard,
    inline_const,
    lazy_cell,
    let_chains,
    map_try_insert,
    never_type,
    split_array,
    trait_alias
)]
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
