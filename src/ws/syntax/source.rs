// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::num::{NonZeroU16, NonZeroU32};
use std::ops::Index;
use std::path::{Path, PathBuf};

/// Position is an arbitrary source position, including the line and column
/// numbers, byte offset, and file index.
///
/// ## Limits for Whitespace
///
/// The character and line counts should each fit in 32 bits (up to
/// 4,294,967,295) and the line width and file count each in 16 bits (up to
/// 65,535).
///
/// ### Large Whitespace programs
///
/// The largest Whitespace programs, pulled from data gathered in the
/// [Whitespace Corpus](https://github.com/wspace/corpus):
///
/// | File                            | Bytes/chars | Lines   | Labels | Longest line |
/// | ------------------------------- | ----------- | ------- | ------ | ------------ |
/// | [8cc.c.eir.ws]                  | 7,512,502   | 699,000 | 63,952 | 35           |
/// | [quine-copy.ws]                 | 661,964     | 46,171  | 13     | 74           |
/// | [QR.ws]                         | 629,795     | 78,727  | 0      | 13           |
/// | [xctf-finals-2020-spaceship.ws] | 539,624     | 45,435  | 2,245  | 35           |
/// | [password_checker.ws]           | 510,875     | 43,120  | 2,154  | 35           |
/// | [sk-whitespace.ws]              | 135,462     | 19,259  | 1,240  | 25           |
/// | [rameev.ws]                     | 94,152      | 21,387  | 0      | 10           |
///
/// [8cc.c.eir.ws]: https://github.com/helvm/helma/blob/master/examples/ws/ws/from-elvm/8cc.c.eir.ws
/// [quine-copy.ws]: https://web.archive.org/web/20150612005338/http://compsoc.dur.ac.uk/whitespace/quine-copy.ws
/// [QR.ws]: https://github.com/mame/quine-relay/blob/spoiler/QR.ws
/// [xctf-finals-2020-spaceship.ws]: https://github.com/umutoztunc/whitesymex/blob/main/tests/data/xctf-finals-2020-spaceship.ws
/// [password_checker.ws]: https://github.com/umutoztunc/whitesymex/blob/main/tests/data/password_checker.ws
/// [sk-whitespace.ws]: https://github.com/kspalaiologos/cosmopolitan-sk/blob/master/sk-whitespace.ws
/// [rameev.ws]: https://gist.github.com/pik4ez/8274216220511d0e42de7881eca782da
///
/// ### Line width limit
///
/// Due to the instruction encoding scheme, the absolute line width limit can be
/// approximated:
///
/// `push` is terminated with an L and can be preceded on the same line with any
/// number of instructions that do not contain L, that is, `add`, `sub`, `div`,
/// `mod`, `store`, `retrieve`, and `shuffle`. This subset of instructions would
/// not be very useful to repeat excessively in sequence, though `retrieve` and
/// `shuffle` could be arbitrarily repeated without requiring a large stack. 128
/// of these instructions followed by a 4096-bit `push` would be up to 4611
/// tokens on a line.
///
/// Labeled control flow instructions contain an L in the opcode and one
/// terminating the label, so always span a full line. With the convention of
/// representing textual labels using eight `S`/`T` tokens per byte and a
/// pathological 2048-byte label (the identifier length limit in some C++
/// compilers), that would be 16384 tokens for the label, plus three more for
/// the opcode.
///
/// `slide` also spans a full line, but stack sizes are usually small and its
/// argument length is logarithmic to the size of the stack, so it wouldn't be
/// more than a few tokens long.
///
/// Comments are not similarly constrained and could be arbitrarily long, but
/// should not occur more than a ratio of 10:1 with `S`/`T`/`L` tokens, even in
/// steganographic programs.
///
/// ### File count limit
///
/// There should be no case where over 65,535 files would be compiled at once.
/// Even the Linux kernel has less overall files than that.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub offset: u32,
    pub line: NonZeroU32,
    pub col: NonZeroU16,
    pub file: FileId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSet {
    files: Vec<File>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    path: PathBuf,
    src: Vec<u8>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u16);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePosition<'a> {
    file: &'a File,
    pos: Position,
}

impl FileSet {
    #[inline]
    pub fn add(&mut self, file: File) -> FileId {
        let id = FileId(self.files.len().try_into().expect("file id overflow"));
        self.files.push(file);
        id
    }

    #[inline]
    pub fn add_from_path(&mut self, path: PathBuf) -> io::Result<FileId> {
        let src = fs::read(&path)?;
        Ok(self.add(File::new(path, src)))
    }

    #[inline]
    #[must_use]
    pub fn position(&self, pos: Position) -> FilePosition<'_> {
        FilePosition { file: &self[pos.file], pos }
    }
}

impl Index<FileId> for FileSet {
    type Output = File;

    #[inline]
    fn index(&self, id: FileId) -> &Self::Output {
        &self.files[id.0 as usize]
    }
}

impl File {
    #[inline]
    #[must_use]
    pub const fn new(path: PathBuf, src: Vec<u8>) -> Self {
        File { path, src }
    }

    #[inline]
    #[must_use]
    pub fn src(&self) -> &[u8] {
        self.src.as_slice()
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Display for FilePosition<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{}:{}", self.file.path, self.pos.line, self.pos.col)
    }
}
