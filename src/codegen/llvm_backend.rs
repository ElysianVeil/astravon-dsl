// src/codegen/llvm_backend.rs

use crate::ast::nodes::{Program, Statement, Expression};
use inkwell::context::Context;  // Inkwell = Rust LLVM bindings
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum};
// use inkwell::execution_engine::ExecutionEngine;
use inkwell::targets::{InitializationConfig, Target};
// use inkwell::AddressSpace;

pub struct LLVMBackend<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> LLVMBackend<'ctx> {
    pub fn new(name: &str, context: &'ctx Context) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        Self { context, module, builder }
    }

    pub fn compile_program(&mut self, program: &Program) {
        // Create the 'main' function
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let main_func = self.module.add_function("main", fn_type, None);

        // Create entry block and position builder there
        let entry_block = self.context.append_basic_block(main_func, "entry");
        self.builder.position_at_end(entry_block);

        // Compile each statement into LLVM IR
        for stmt in &program.statements {
            let _ = self.compile_statement(stmt);
        }

        // Return 0 from main
        let _ = self.builder.build_return(Some(&i32_type.const_int(0, false)));
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<BasicValueEnum<'ctx>, String> {
        match stmt {
            Statement::Print(expr) => {
                let c_string_type = self.context.i8_type().ptr_type(0.into());

                // Declare external "puts" function if not already declared
                let puts_func = self.module.get_function("puts").unwrap_or_else(|| {
                    let puts_type = self.context.i32_type().fn_type(&[c_string_type.into()], false);
                    self.module.add_function("puts", puts_type, None)
                });

                // Compile expression to LLVM value
                let string_val = self.compile_expression(&Expression::Literal(expr.clone()))    
                    .map_err(|e| format!("Error compiling expression in Print: {}", e))?;

                // Build call to puts
                let call = self.builder.build_call(puts_func, &[string_val.into()], "call_puts")
                    .expect("LLVM build_call failed");
                Ok(call.try_as_basic_value().left().unwrap_or_else(|| {
                    panic!("puts() did not return a BasicValueEnum")
                }))
            // Add other statement kinds here
            }
        }
    }
    



    fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::Literal(text) => {
                // Generate a unique name for each string literal
                static mut COUNTER: usize = 0;
                let name = unsafe {
                    let n = COUNTER;
                    COUNTER += 1;
                    format!("str_{}", n)
                };

                let gv = self.builder.build_global_string_ptr(text, &name)
                    .map_err(|_| format!("Failed to create global string literal '{}'", text))?;
                Ok(gv.as_pointer_value().into())
            }
        }
    }


    pub fn write_ir_to_file(&self, path: &str) {
        self.module.print_to_file(path).unwrap();
    }
}

pub fn initialize_llvm() {
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");
}

pub fn generate(ast: &Program) -> Result<Vec<u8>, String> {
    let context = Context::create();
    let mut backend = super::llvm_backend::LLVMBackend::new("module", &context);

    // Compile the program to LLVM IR
    backend.compile_program(ast);

    // Optionally, write IR to file for inspection
    backend.write_ir_to_file("output.ll");

    // --- AVM bytecode path ---
    let mut bytecode = Vec::new();

    for stmt in &ast.statements {
        match stmt {
            Statement::Print(text) => {
                // 0x01 = PushStr (custom opcode)
                bytecode.push(0x01);
                let bytes = text.as_bytes();
                bytecode.push(bytes.len() as u8); // length prefix
                bytecode.extend_from_slice(bytes);
                // Example AVM opcode for Print: 0x02
                bytecode.push(0x02);
                // Optionally: append string bytes or an index if using a string table
                // Here we keep it simple
            }
            // _ => {}
            // Add more statement kinds here
        }
    }

    Ok(bytecode)
}