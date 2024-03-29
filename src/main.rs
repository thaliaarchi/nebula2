// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use clap::{Args, Parser as CliParser, Subcommand};
use nebula2::ws::{
    inst::{Feature, Features, Inst, InstArg, InstError},
    parse::Parser,
    syntax::{IntLiteral, LabelLiteral},
    token::{bit_unpack_dynamic, lex_mapping, BitOrderDynamic, Lexer, Mapping, MappingLexer},
};

#[derive(Debug, CliParser)]
#[command(author, version, about, long_about = None)]
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
    #[arg(required = true)]
    filename: PathBuf,
    /// Disable UTF-8 validation
    #[arg(long, default_value_t = false)]
    ascii: bool,
    /// Set the bit order for bit packing
    #[arg(long, default_value_t = BitOrderDynamic::Msb0)]
    bit_order: BitOrderDynamic,
    /// Set the mapping for S
    #[arg(long)]
    mapping_s: Option<String>,
    /// Set the mapping for T
    #[arg(long)]
    mapping_t: Option<String>,
    /// Set the mapping for L
    #[arg(long)]
    mapping_l: Option<String>,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Command::Disasm(program) => disassemble(program),
        Command::Features(program) => detect_features(program),
    }
}

fn parse(program: ProgramOptions) -> Parser<'static, Box<dyn Lexer>> {
    let src = fs::read(&program.filename).unwrap();
    let ext = program.filename.extension().and_then(OsStr::to_str);
    let lex: Box<dyn Lexer> = if ext == Some("wsx") {
        Box::new(
            bit_unpack_dynamic(&src, program.bit_order)
                .into_iter()
                .map(Ok),
        )
    } else {
        // TODO: Avoid leaking
        let src: &'static Vec<u8> = Box::leak(Box::new(src));
        if program.mapping_s != None || program.mapping_t != None || program.mapping_l != None {
            lex_mapping(
                src,
                program.mapping_s.expect("empty S").into(),
                program.mapping_t.expect("empty T").into(),
                program.mapping_l.expect("empty L").into(),
                program.ascii,
                true,
            )
            .expect("invalid mapping")
        } else if program.ascii {
            Box::new(MappingLexer::new_bytes(src, Mapping::default()))
        } else {
            Box::new(MappingLexer::new_utf8(src, Mapping::default(), true))
        }
    };
    Parser::new(lex)
}

fn disassemble(program: ProgramOptions) {
    for inst in parse(program) {
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
    let mut features = Features::empty();
    for inst in parse(program) {
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
