//! Astravon DSL Core Library
//! 
//! This crate exposes the building blocks of the Astravon DSL
//! so it can be embedded in other Rust applications or compiled
//! into the Astravon Virtual Machine (AVM).

// Make your internal modules visible here
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod semantics;
pub mod codegen;
pub mod runtime;
pub mod stdlib;

// Re-exports for easy use
pub use lexer::*;
pub use parser::*;
pub use runtime::*;

// Example usage in lib.rs after parsing
use crate::semantics::analyzer::Analyzer;
use crate::semantics::analyzer::SemanticError;
// use runtime::OpCode;


impl From<ParserError> for String {
    fn from(e: ParserError) -> Self {
        format!("{:?}", e)
    }
}

impl From<SemanticError> for String {
    fn from(e: SemanticError) -> Self {
        format!("{:?}", e)
    }
}


/// Version info for the DSL
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compile an Astravon DSL source string to bytecode with stage logging.
pub fn compile_source(source: &str) -> Result<Vec<u8>, String> {
    println!("🚀 Starting compilation...");

    // 1️⃣ Tokenization
    println!("1️⃣ Tokenizing source...");
    let mut tokens = match lexer::tokenizer::Lexer::tokenize(source) {
        Ok(t) => {
            println!("✅ Tokenization successful: {} tokens", t.len());
            t
        }
        Err(e) => {
            eprintln!("❌ Tokenization error: {}", e);
            return Err(format!("Tokenization failed: {}", e));
        }
    };

    // 1b. strip comments
    tokens = strip_comments(tokens);

    // 2️⃣ Parsing
    println!("2️⃣ Parsing tokens into AST...");
    let ast = match parser::grammar::parse_program(tokens) {
        Ok(ast) => {
            println!("✅ Parsing successful: {} statements", ast.statements.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Parsing error: {:?}", e);
            return Err(format!("Parsing failed: {:?}", e));
        }
    };

    // 3️⃣ Semantic Analysis
    println!("3️⃣ Performing semantic analysis...");
    let mut analyzer = Analyzer::new();
    if let Err(e) = analyzer.analyze_program(&ast) {
        eprintln!("❌ Semantic error: {:?}", e);
        return Err(format!("Semantic analysis failed: {:?}", e));
    }
    println!("✅ Semantic analysis passed");

    // 4️⃣ Codegen
    println!("4️⃣ Generating bytecode...");
    let bytecode = match codegen::llvm_backend::generate(&ast) {
        Ok(bytes) => {
            println!("✅ Bytecode generated: {} bytes", bytes.len());
            bytes
        }
        Err(e) => {
            eprintln!("❌ Codegen error: {:?}", e);
            return Err(format!("Codegen failed: {:?}", e));
        }
    };

    println!("🎉 Compilation finished successfully!");
    Ok(bytecode)
}

/// Run an Astravon DSL source string directly with stage logging.
pub fn run_source(source: &str) -> Result<(), String> {
    println!("🚀 Running source in interpret mode...");

    // Compile first (includes tokenization, parsing, semantic, codegen)
    let bytecode = compile_source(source)?;

    // Decode bytecode for the VM
    fn decode_bytecode(bytes: &[u8]) -> Vec<OpCode> {
        let mut ops = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                0x01 => { // PushStr
                    let len = bytes[i + 1] as usize;
                    let s = String::from_utf8(bytes[i + 2 .. i + 2 + len].to_vec())
                        .expect("Invalid UTF-8 in string literal");
                    ops.push(OpCode::PushStr(s));
                    i += 2 + len;
                }
                0x02 => { // Print
                    ops.push(OpCode::Print);
                    i += 1;
                }
                _ => panic!("Unknown opcode 0x{:02X}", bytes[i]),
            }
        }
        ops
    }

    println!("5️⃣ Decoding bytecode for VM...");
    let opcodes = decode_bytecode(&bytecode);
    println!("✅ {} opcodes decoded", opcodes.len());

    // Execute
    println!("6️⃣ Executing on AVM...");
    let mut vm = runtime::AVM::new(opcodes);
    let _ = runtime::executor::execute(&mut vm, bytecode);

    println!("🎉 Execution finished successfully!");
    Ok(())
}

pub fn strip_comments(mut tokens: Vec<Token>) -> Vec<Token> {
    let mut filtered = Vec::new();
    let mut skip_line = false;
    let mut current_line = 0;

    for token in tokens {
        // detect start of a comment
        if let TokenKind::Symbol('#') = token.kind {
            skip_line = true;
            current_line = token.line;
            continue;
        }

        // reset skip_line when new line reached
        if skip_line && token.line != current_line {
            skip_line = false;
        }

        // push only non-comment tokens
        if !skip_line {
            filtered.push(token);
        }
    }

    filtered
}
