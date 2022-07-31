// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

#![feature(
    box_patterns,
    box_syntax,
    concat_bytes,
    const_array_into_iter_constructors,
    const_clone,
    const_convert,
    const_default_impls,
    const_for,
    const_intoiterator_identity,
    const_mut_refs,
    const_option,
    const_option_cloned,
    const_option_ext,
    const_trait_impl,
    core_intrinsics,
    if_let_guard,
    inline_const,
    int_log,
    label_break_value,
    let_chains,
    let_else,
    map_try_insert,
    never_type,
    split_array,
    trait_alias
)]
#![deny(clippy::pedantic)]

pub mod bf;
pub mod syntax;
pub mod text;
pub mod ws;
