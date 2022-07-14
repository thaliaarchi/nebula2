// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

#![feature(box_syntax)]

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use bitvec::order::Msb0;
use clap::{Args, Parser as CliParser, Subcommand};
use nebula2::ws::{
    inst::{Feature, Features, Inst, InstArg, InstError},
    parse::{ParseTable, Parser},
    syntax::{IntLiteral, LabelLiteral},
    token::{bit_unpack_aligned, Lexer, Mapping, MappingLexer},
};

#[derive(Debug, CliParser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Disassemble the program to Whitespace assembly syntax.
    Disasm(ProgramOptions),
    /// Detect the spec version (0.2 or 0.3) for a program and any non-standard
    /// instructions
    Features(ProgramOptions),
}

#[derive(Debug, Args)]
struct ProgramOptions {
    /// Path to Whitespace program
    #[clap(required = true, value_parser)]
    filename: PathBuf,
    /// Disable UTF-8 validation
    #[clap(long, value_parser, default_value_t = false)]
    ascii: bool,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Command::Disasm(program) => disassemble(program),
        Command::Features(program) => detect_features(program),
    }
}

// TODO: Structure better with lifetimes
macro_rules! get_parser(
    ($parser:ident, $program:ident) => {
        let src = fs::read(&$program.filename).unwrap();
        let ext = $program.filename.extension().map(OsStr::to_str).flatten();
        let lex: Box<dyn Lexer> = match ext {
            Some("wsx") => box bit_unpack_aligned::<u8, Msb0>(&src).into_iter().map(Ok),
            _ if $program.ascii => box MappingLexer::new_bytes(&src, Mapping::default()),
            _ => box MappingLexer::new_utf8(&src, Mapping::default(), true),
        };
        let table = ParseTable::with_all();
        let $parser = Parser::new(&table, lex);
    }
);

fn disassemble(program: ProgramOptions) {
    get_parser!(parser, program);
    for inst in parser {
        if let Inst::Error(err) = inst {
            println!("error: {err:?}");
        } else {
            let inst = inst.map_arg(|_, arg| -> Result<_, InstError> {
                match arg {
                    InstArg::Int(n) => Ok(InstArg::Int(IntLiteral::from(n))),
                    InstArg::Label(l) => Ok(InstArg::Label(LabelLiteral::from_bits(l))),
                }
            });
            println!("{inst}");
        }
    }
}

fn detect_features(program: ProgramOptions) {
    get_parser!(parser, program);
    let mut features = Features::empty();
    for inst in parser {
        if let Inst::Error(err) = inst {
            println!("error: {err:?}");
        } else if let Some(feature) = inst.opcode().feature() {
            features.insert(feature);
        }
    }
    println!("Features:");
    if !features.contains(Feature::Wspace0_3) {
        println!("- wspace 0.2");
    }
    for feature in features {
        println!("- {feature}");
    }
}
