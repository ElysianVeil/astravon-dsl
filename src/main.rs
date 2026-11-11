//! Astravon DSL CLI
//!
//! This is the entry point for the Astravon DSL compiler/interpreter.
//! It reads a `.avon` source file, compiles it (or interprets it),
//! and either executes it on the AVM or outputs bytecode.

use astravon_dsl::{VERSION, compile_source, run_source};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Astravon DSL v{}", VERSION);
        eprintln!(
            "Usage: {} <file.avon> [--compile-only] [--output <file>]",
            args[0]
        );
        std::process::exit(1);
    }

    // Options
    let mut compile_only = false;
    let mut output_file: Option<String> = None;

    // Parse args
    let mut i = 1;
    let mut filename: Option<String> = None;
    while i < args.len() {
        match args[i].as_str() {
            "--compile-only" => compile_only = true,
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Missing output filename after --output");
                    std::process::exit(1);
                }
            }
            _ => {
                // first non-flag argument is the file name
                if filename.is_none() {
                    filename = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    let filename = filename.expect("No input file specified");

    let source = fs::read_to_string(&filename).unwrap_or_else(|_| {
        eprintln!("Could not read file: {}", filename);
        std::process::exit(1);
    });

    if compile_only {
        match compile_source(&source) {
            Ok(bytecode) => {
                if let Some(out) = output_file {
                    fs::write(&out, &bytecode).expect("Failed to write bytecode");
                    println!("✅ Bytecode written to {}", out);
                } else {
                    println!(
                        "✅ Compilation succeeded ({} bytes of bytecode)",
                        bytecode.len()
                    );
                }
            }
            Err(e) => {
                eprintln!("❌ Compilation error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Interpret/run immediately
        if let Err(e) = run_source(&source) {
            eprintln!("❌ Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}
