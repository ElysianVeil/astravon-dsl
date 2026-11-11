use crate::ast::nodes::{Expression, Program, Statement};

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::values::BasicValueEnum;

pub struct LLVMBackend<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> LLVMBackend<'ctx> {
    pub fn new(name: &str, context: &'ctx Context) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let function_type = i32_type.fn_type(&[], false);

        let main_function = self.module.add_function("main", function_type, None);

        let entry_block = self.context.append_basic_block(main_function, "entry");

        self.builder.position_at_end(entry_block);

        for statement in &program.statements {
            self.compile_statement(statement)?;
        }

        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .map_err(|error| format!("Failed to build return instruction: {:?}", error))?;

        Ok(())
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Print(expression) => {
                let string_type = self.context.i8_type().ptr_type(0.into());

                // Declare puts if it does not already exist.
                let puts_function = self.module.get_function("puts").unwrap_or_else(|| {
                    let puts_type = self
                        .context
                        .i32_type()
                        .fn_type(&[string_type.into()], false);

                    self.module.add_function("puts", puts_type, None)
                });

                let value = self.compile_expression(expression)?;

                self.builder
                    .build_call(puts_function, &[value.into()], "call_puts")
                    .map_err(|error| format!("Failed to build puts call: {:?}", error))?;

                Ok(())
            }
        }
    }

    fn compile_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Convert the current primitive expression into printable text.
        let text = match expression {
            Expression::StringLiteral(value) => value.clone(),
            Expression::IntegerLiteral(value) => value.to_string(),
            Expression::FloatLiteral(value) => value.to_string(),
            Expression::BooleanLiteral(value) => value.to_string(),
            Expression::CharacterLiteral(value) => value.to_string(),
            Expression::NullLiteral => "null".to_string(),
        };

        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let name = format!("str_{}", COUNTER.fetch_add(1, Ordering::Relaxed));

        let global_string = self
            .builder
            .build_global_string_ptr(&text, &name)
            .map_err(|error| format!("Failed to create global string: {:?}", error))?;

        Ok(global_string.as_pointer_value().into())
    }

    pub fn ir_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn write_ir_to_file(&self, path: &str) -> Result<(), String> {
        self.module
            .print_to_file(path)
            .map_err(|error| format!("Failed to write LLVM IR to {}: {:?}", path, error))
    }
}

pub fn initialize_llvm() {
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native LLVM target");
}

/// Generate LLVM Intermediate Representation from an AstravonDSL AST.
pub fn generate_ir(ast: &Program) -> Result<String, String> {
    let context = Context::create();

    let mut backend = LLVMBackend::new("astravon_module", &context);

    backend.compile_program(ast)?;

    Ok(backend.ir_string())
}
