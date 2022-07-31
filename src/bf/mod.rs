// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

pub mod ook;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// `>` or `Ook. Ook?`
    Right,
    /// `<` or `Ook? Ook.`
    Left,
    /// `+` or `Ook. Ook.`
    Inc,
    /// `-` or `Ook! Ook!`
    Dec,
    /// `.` or `Ook! Ook.`
    Output,
    /// `,` or `Ook. Ook!`
    Input,
    /// `[` or `Ook! Ook?`
    Head,
    /// `]` or `Ook? Ook!`
    Tail,
    /// `Ook? Ook?`
    Nop,
}
