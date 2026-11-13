use std::collections::HashMap;

use crate::ast::nodes::{
    BinaryOperator, Expression, MatchArm, MatchPattern, Program, Statement, StringSegment,
    UnaryOperator,
};
use crate::runtime::OpCode;

type UnresolvedCall = (usize, String, usize);

struct LoopContext {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
    continue_target: Option<usize>,
    scope_depth: usize,
}

/// Generate AVM instructions from an Astravon AST.
pub fn compile_program(program: &Program) -> Result<Vec<OpCode>, String> {
    let mut instructions = Vec::new();
    let mut function_entries = HashMap::new();
    let mut unresolved_calls: Vec<UnresolvedCall> = Vec::new();
    let mut loop_stack = Vec::new();
    let mut scope_depth = 0;

    // Compile top-level executable statements first.
    for statement in &program.statements {
        if !matches!(statement, Statement::Function { .. }) {
            compile_statement(
                statement,
                &mut instructions,
                &mut unresolved_calls,
                &mut loop_stack,
                &mut scope_depth,
            )?;
        }
    }

    // Skip function bodies during normal program execution.
    let skip_functions_index = instructions.len();
    instructions.push(OpCode::Jump(usize::MAX));

    // Compile function bodies and record their entry points.
    for statement in &program.statements {
        if let Statement::Function {
            name,
            parameters,
            body,
            ..
        } = statement
        {
            if function_entries.contains_key(name) {
                return Err(format!("Duplicate function definition: {}", name));
            }

            let entry = instructions.len();
            function_entries.insert(name.clone(), entry);

            // Arguments are pushed left-to-right.
            // Bind them in reverse order because the operand stack is LIFO.
            for parameter in parameters.iter().rev() {
                instructions.push(OpCode::DefineVar(parameter.name.clone()));
            }

            let mut function_loop_stack = Vec::new();
            let mut function_scope_depth = 0;

            for body_statement in body {
                compile_statement(
                    body_statement,
                    &mut instructions,
                    &mut unresolved_calls,
                    &mut function_loop_stack,
                    &mut function_scope_depth,
                )?;
            }

            // Safe fallback for a function that reaches its end.
            instructions.push(OpCode::PushNull);
            instructions.push(OpCode::Return);
        }
    }

    // Patch the jump that skips all function bodies.
    let program_end = instructions.len();
    instructions[skip_functions_index] = OpCode::Jump(program_end);

    // Resolve function calls, including forward and recursive calls.
    for (instruction_index, function_name, argument_count) in unresolved_calls {
        let target = function_entries
            .get(&function_name)
            .copied()
            .ok_or_else(|| format!("Undefined function: {}", function_name))?;

        instructions[instruction_index] = OpCode::Call {
            target,
            argument_count,
        };
    }

    Ok(instructions)
}

fn compile_statement(
    statement: &Statement,
    instructions: &mut Vec<OpCode>,
    unresolved_calls: &mut Vec<UnresolvedCall>,
    loop_stack: &mut Vec<LoopContext>,
    scope_depth: &mut usize,
) -> Result<(), String> {
    match statement {
        Statement::Let {
            name, initializer, ..
        } => {
            compile_expression(initializer, instructions, unresolved_calls)?;
            instructions.push(OpCode::DefineVar(name.clone()));
        }

        Statement::Assignment { name, value } => {
            compile_expression(value, instructions, unresolved_calls)?;
            instructions.push(OpCode::StoreVar(name.clone()));
        }

        Statement::Print(expression) => {
            compile_expression(expression, instructions, unresolved_calls)?;
            instructions.push(OpCode::Print);
        }

        Statement::Expression(expression) => {
            compile_expression(expression, instructions, unresolved_calls)?;
            instructions.push(OpCode::Pop);
        }

        Statement::Return(expression) => {
            if let Some(expression) = expression {
                compile_expression(expression, instructions, unresolved_calls)?;
            } else {
                instructions.push(OpCode::PushNull);
            }

            instructions.push(OpCode::Return);
        }

        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            // Evaluate the condition.
            compile_expression(condition, instructions, unresolved_calls)?;

            // Jump to the else branch (or end) when the condition is false.
            let false_jump_index = instructions.len();
            instructions.push(OpCode::JumpIfFalse(usize::MAX));

            // True branch: create an independent lexical scope.
            instructions.push(OpCode::EnterScope);
            *scope_depth += 1;
            for branch_statement in then_branch {
                compile_statement(
                    branch_statement,
                    instructions,
                    unresolved_calls,
                    loop_stack,
                    scope_depth,
                )?;
            }

            instructions.push(OpCode::ExitScope);
            *scope_depth -= 1;

            if let Some(else_branch) = else_branch {
                // Skip the else branch after executing the true branch.
                let end_jump_index = instructions.len();
                instructions.push(OpCode::Jump(usize::MAX));

                // False condition enters the else branch here.
                let else_start = instructions.len();
                instructions[false_jump_index] = OpCode::JumpIfFalse(else_start);

                // Else branch gets its own independent lexical scope.
                instructions.push(OpCode::EnterScope);
                *scope_depth += 1;

                for branch_statement in else_branch {
                    compile_statement(
                        branch_statement,
                        instructions,
                        unresolved_calls,
                        loop_stack,
                        scope_depth,
                    )?;
                }

                instructions.push(OpCode::ExitScope);
                *scope_depth -= 1;

                // Both branches converge here.
                let end = instructions.len();
                instructions[end_jump_index] = OpCode::Jump(end);
            } else {
                // No else branch: false conditions skip the entire true branch.
                let end = instructions.len();
                instructions[false_jump_index] = OpCode::JumpIfFalse(end);
            }
        }

        Statement::While { condition, body } => {
            let loop_start = instructions.len();

            compile_expression(condition, instructions, unresolved_calls)?;

            let exit_jump_index = instructions.len();

            instructions.push(OpCode::JumpIfFalse(usize::MAX));

            // Record loop context before entering the body scope.
            loop_stack.push(LoopContext {
                break_jumps: Vec::new(),
                continue_jumps: Vec::new(),
                continue_target: Some(loop_start),
                scope_depth: *scope_depth,
            });

            instructions.push(OpCode::EnterScope);
            *scope_depth += 1;

            for body_statement in body {
                compile_statement(
                    body_statement,
                    instructions,
                    unresolved_calls,
                    loop_stack,
                    scope_depth,
                )?;
            }

            instructions.push(OpCode::ExitScope);
            *scope_depth -= 1;

            instructions.push(OpCode::Jump(loop_start));

            let loop_end = instructions.len();

            instructions[exit_jump_index] = OpCode::JumpIfFalse(loop_end);

            // Patch all breaks belonging to this loop.
            let context = loop_stack.pop().unwrap();

            for jump_index in context.break_jumps {
                instructions[jump_index] = OpCode::Jump(loop_end);
            }
        }

        Statement::For {
            variable,
            iterable,
            body,
        } => {
            compile_for_statement(
                variable,
                iterable,
                body,
                instructions,
                unresolved_calls,
                loop_stack,
                scope_depth,
            )?;
        }

        Statement::Break => {
            let context = loop_stack
                .last_mut()
                .ok_or_else(|| "Internal compiler error: break outside loop".to_string())?;

            // Clean up lexical scopes exited by this break.
            for _ in context.scope_depth..*scope_depth {
                instructions.push(OpCode::ExitScope);
            }

            let jump_index = instructions.len();

            instructions.push(OpCode::Jump(usize::MAX));

            context.break_jumps.push(jump_index);
        }

        Statement::Continue => {
            let context = loop_stack
                .last_mut()
                .ok_or_else(|| "Internal compiler error: continue outside loop".to_string())?;

            // Clean up lexical scopes exited by this continue.
            for _ in context.scope_depth..*scope_depth {
                instructions.push(OpCode::ExitScope);
            }

            if let Some(target) = context.continue_target {
                instructions.push(OpCode::Jump(target));
            } else {
                let jump_index = instructions.len();

                instructions.push(OpCode::Jump(usize::MAX));

                context.continue_jumps.push(jump_index);
            }
        }

        Statement::Function { name, .. } => {
            return Err(format!(
                "Nested function declarations are not supported: {}",
                name
            ));
        }
    }

    Ok(())
}

/// Compile a for-in loop into AVM instructions.
fn compile_for_statement(
    variable: &str,
    iterable: &Expression,
    body: &[Statement],
    instructions: &mut Vec<OpCode>,
    unresolved_calls: &mut Vec<UnresolvedCall>,
    loop_stack: &mut Vec<LoopContext>,
    scope_depth: &mut usize,
) -> Result<(), String> {
    // Generate internal names that cannot collide with source identifiers.
    // The NUL character cannot occur in a normal Astravon identifier.
    let loop_id = instructions.len();

    let collection_name = format!("\0astravon::for::{loop_id}::collection");

    let index_name = format!("\0astravon::for::{loop_id}::index");

    // Create a scope for compiler-generated loop variables.
    instructions.push(OpCode::EnterScope);
    *scope_depth += 1;

    // Remember the scope depth before entering this loop.
    let loop_scope_depth = *scope_depth;

    // Evaluate and store the iterable exactly once.
    compile_expression(iterable, instructions, unresolved_calls)?;

    instructions.push(OpCode::DefineVar(collection_name.clone()));

    // Initialize the loop index.
    instructions.push(OpCode::PushInt(0));

    instructions.push(OpCode::DefineVar(index_name.clone()));

    // Beginning of the loop condition.
    let loop_start = instructions.len();

    // Check: index < collection.length
    instructions.push(OpCode::LoadVar(index_name.clone()));

    instructions.push(OpCode::LoadVar(collection_name.clone()));

    instructions.push(OpCode::ArrayLength);

    instructions.push(OpCode::LessThan);

    // Placeholder for the loop exit destination.
    let exit_jump = instructions.len();

    instructions.push(OpCode::JumpIfFalse(usize::MAX));

    // Register this loop so break and continue can target it.
    loop_stack.push(LoopContext {
        break_jumps: Vec::new(),
        continue_jumps: Vec::new(),
        continue_target: None,
        scope_depth: loop_scope_depth,
    });

    // Create a fresh scope for the current iteration.
    instructions.push(OpCode::EnterScope);
    *scope_depth += 1;

    // Retrieve collection[index].
    instructions.push(OpCode::LoadVar(collection_name.clone()));

    instructions.push(OpCode::LoadVar(index_name.clone()));

    instructions.push(OpCode::ArrayGet);

    // Declare the loop variable.
    instructions.push(OpCode::DefineVar(variable.to_string()));

    // Give the loop body its own lexical scope.
    instructions.push(OpCode::EnterScope);
    *scope_depth += 1;

    for statement in body {
        compile_statement(
            statement,
            instructions,
            unresolved_calls,
            loop_stack,
            scope_depth,
        )?;
    }

    // End the body scope.
    instructions.push(OpCode::ExitScope);
    *scope_depth -= 1;

    // End the current iteration's variable scope.
    instructions.push(OpCode::ExitScope);
    *scope_depth -= 1;

    // Continue jumps here, before the index increment.
    let continue_target = instructions.len();

    let context = loop_stack.last_mut().unwrap();

    context.continue_target = Some(continue_target);

    // Patch all continue statements belonging to this loop.
    for jump_index in &context.continue_jumps {
        instructions[*jump_index] = OpCode::Jump(continue_target);
    }

    // Increment the index.
    instructions.push(OpCode::LoadVar(index_name.clone()));

    instructions.push(OpCode::PushInt(1));

    instructions.push(OpCode::Add);

    instructions.push(OpCode::StoreVar(index_name));

    // Return to the loop condition.
    instructions.push(OpCode::Jump(loop_start));

    // Patch the conditional jump to the instruction after the loop.
    let loop_exit = instructions.len();

    instructions[exit_jump] = OpCode::JumpIfFalse(loop_exit);

    // Resolve break statements belonging to this loop.
    let context = loop_stack.pop().unwrap();

    for jump_index in context.break_jumps {
        instructions[jump_index] = OpCode::Jump(loop_exit);
    }

    // Remove compiler-generated variables.
    instructions.push(OpCode::ExitScope);
    *scope_depth -= 1;

    Ok(())
}

/// Compile a match expression into AVM instructions.
///
/// The scrutinee is evaluated exactly once and stored in a
/// compiler-generated lexical variable. Each arm tests its pattern
/// against that value. A successful arm leaves exactly one result
/// on the operand stack.
fn compile_match_expression(
    value: &Expression,
    arms: &[MatchArm],
    instructions: &mut Vec<OpCode>,
    unresolved_calls: &mut Vec<UnresolvedCall>,
) -> Result<(), String> {
    if arms.is_empty() {
        return Err("Internal compiler error: match expression has no arms".to_string());
    }

    let match_id = instructions.len();
    let value_name = format!("\0astravon::match::{match_id}::value");

    // Create an isolated scope for the temporary match variable.
    instructions.push(OpCode::EnterScope);

    // Evaluate the match value exactly once.
    compile_expression(value, instructions, unresolved_calls)?;

    instructions.push(OpCode::DefineVar(value_name.clone()));

    let mut end_jumps = Vec::new();

    for arm in arms {
        // Compile the pattern into a Boolean result.
        compile_match_pattern(&arm.pattern, &value_name, instructions)?;

        // If the pattern does not match, continue to the next arm.
        let pattern_failure_jump = instructions.len();

        instructions.push(OpCode::JumpIfFalse(usize::MAX));

        // Evaluate an optional guard only after the pattern matches.
        let guard_failure_jump = if let Some(guard) = &arm.guard {
            compile_expression(guard, instructions, unresolved_calls)?;

            let jump_index = instructions.len();

            instructions.push(OpCode::JumpIfFalse(usize::MAX));

            Some(jump_index)
        } else {
            None
        };

        // Pattern and guard succeeded. Evaluate the arm result.
        compile_expression(&arm.expression, instructions, unresolved_calls)?;

        // Skip all remaining arms.
        let end_jump_index = instructions.len();

        instructions.push(OpCode::Jump(usize::MAX));

        end_jumps.push(end_jump_index);

        // Both pattern failure and guard failure continue here.
        let next_arm = instructions.len();

        instructions[pattern_failure_jump] = OpCode::JumpIfFalse(next_arm);

        if let Some(jump_index) = guard_failure_jump {
            instructions[jump_index] = OpCode::JumpIfFalse(next_arm);
        }
    }

    // Semantic analysis should guarantee exhaustiveness.
    // Retain a safe fallback if compilation is invoked independently.
    instructions.push(OpCode::PushNull);

    // All successful arms converge here.
    let match_end = instructions.len();

    for jump_index in end_jumps {
        instructions[jump_index] = OpCode::Jump(match_end);
    }

    // Remove the temporary match-value scope.
    // The selected result remains on the operand stack.
    instructions.push(OpCode::ExitScope);

    Ok(())
}

/// Compile a match pattern into a Boolean value.
///
/// Supported patterns:
/// - Literal patterns
/// - Wildcard patterns
/// - OR patterns
/// - Inclusive numeric ranges
/// - Exclusive numeric ranges
fn compile_match_pattern(
    pattern: &MatchPattern,
    value_name: &str,
    instructions: &mut Vec<OpCode>,
) -> Result<(), String> {
    match pattern {
        MatchPattern::Int(value) => {
            instructions.push(OpCode::LoadVar(value_name.to_string()));
            instructions.push(OpCode::PushInt(*value));
            instructions.push(OpCode::Equal);
        }

        MatchPattern::Float(value) => {
            instructions.push(OpCode::LoadVar(value_name.to_string()));
            instructions.push(OpCode::PushFloat(*value));
            instructions.push(OpCode::Equal);
        }

        MatchPattern::Bool(value) => {
            instructions.push(OpCode::LoadVar(value_name.to_string()));
            instructions.push(OpCode::PushBool(*value));
            instructions.push(OpCode::Equal);
        }

        MatchPattern::Char(value) => {
            instructions.push(OpCode::LoadVar(value_name.to_string()));
            instructions.push(OpCode::PushChar(*value));
            instructions.push(OpCode::Equal);
        }

        MatchPattern::String(value) => {
            instructions.push(OpCode::LoadVar(value_name.to_string()));
            instructions.push(OpCode::PushString(value.clone()));
            instructions.push(OpCode::Equal);
        }

        MatchPattern::Wildcard => {
            instructions.push(OpCode::PushBool(true));
        }

        MatchPattern::Or(patterns) => {
            if patterns.is_empty() {
                return Err("Internal compiler error: empty OR pattern".to_string());
            }

            let mut success_jumps = Vec::new();

            for alternative in patterns {
                compile_match_pattern(alternative, value_name, instructions)?;

                // Try the next alternative if this one fails.
                let failure_jump = instructions.len();

                instructions.push(OpCode::JumpIfFalse(usize::MAX));

                // An alternative matched.
                instructions.push(OpCode::PushBool(true));

                // Skip the remaining alternatives.
                let success_jump = instructions.len();

                instructions.push(OpCode::Jump(usize::MAX));

                success_jumps.push(success_jump);

                // Continue testing alternatives here.
                let next_alternative = instructions.len();

                instructions[failure_jump] = OpCode::JumpIfFalse(next_alternative);
            }

            // No alternative matched.
            instructions.push(OpCode::PushBool(false));

            let or_end = instructions.len();

            for jump_index in success_jumps {
                instructions[jump_index] = OpCode::Jump(or_end);
            }
        }

        MatchPattern::Range {
            start,
            end,
            inclusive,
        } => {
            // Lower-bound check: value >= start.
            instructions.push(OpCode::LoadVar(value_name.to_string()));

            compile_match_range_boundary(start, instructions)?;

            instructions.push(OpCode::GreaterThanOrEqual);

            // Upper-bound check.
            instructions.push(OpCode::LoadVar(value_name.to_string()));

            compile_match_range_boundary(end, instructions)?;

            if *inclusive {
                instructions.push(OpCode::LessThanOrEqual);
            } else {
                instructions.push(OpCode::LessThan);
            }

            // Both conditions must be true.
            instructions.push(OpCode::And);
        }
    }

    Ok(())
}

/// Compile a numeric range boundary.
fn compile_match_range_boundary(
    boundary: &MatchPattern,
    instructions: &mut Vec<OpCode>,
) -> Result<(), String> {
    match boundary {
        MatchPattern::Int(value) => {
            instructions.push(OpCode::PushInt(*value));
        }

        MatchPattern::Float(value) => {
            instructions.push(OpCode::PushFloat(*value));
        }

        _ => {
            return Err("Internal compiler error: invalid numeric range boundary".to_string());
        }
    }

    Ok(())
}

fn compile_expression(
    expression: &Expression,
    instructions: &mut Vec<OpCode>,
    unresolved_calls: &mut Vec<UnresolvedCall>,
) -> Result<(), String> {
    match expression {
        Expression::StringLiteral(value) => {
            instructions.push(OpCode::PushString(value.clone()));
        }

        Expression::InterpolatedString(segments) => {
            if segments.is_empty() {
                instructions.push(OpCode::PushString(String::new()));
                return Ok(());
            }

            for segment in segments {
                match segment {
                    StringSegment::Text(value) => {
                        instructions.push(OpCode::PushString(value.clone()));
                    }

                    StringSegment::Expression(expression) => {
                        compile_expression(expression, instructions, unresolved_calls)?;
                    }
                }
            }

            instructions.push(OpCode::BuildString(segments.len()));
        }

        Expression::IntegerLiteral(value) => {
            instructions.push(OpCode::PushInt(*value));
        }

        Expression::FloatLiteral(value) => {
            instructions.push(OpCode::PushFloat(*value));
        }

        Expression::BooleanLiteral(value) => {
            instructions.push(OpCode::PushBool(*value));
        }

        Expression::CharacterLiteral(value) => {
            instructions.push(OpCode::PushChar(*value));
        }

        Expression::NullLiteral => {
            instructions.push(OpCode::PushNull);
        }

        Expression::Identifier(name) => {
            instructions.push(OpCode::LoadVar(name.clone()));
        }

        Expression::Call { name, arguments } => {
            for argument in arguments {
                compile_expression(argument, instructions, unresolved_calls)?;
            }

            let instruction_index = instructions.len();

            // The target is patched after all functions have been compiled.
            instructions.push(OpCode::Call {
                target: usize::MAX,
                argument_count: arguments.len(),
            });

            unresolved_calls.push((instruction_index, name.clone(), arguments.len()));
        }

        Expression::Binary {
            left,
            operator,
            right,
        } => {
            compile_expression(left, instructions, unresolved_calls)?;
            compile_expression(right, instructions, unresolved_calls)?;

            instructions.push(match operator {
                BinaryOperator::Add => OpCode::Add,
                BinaryOperator::Subtract => OpCode::Subtract,
                BinaryOperator::Multiply => OpCode::Multiply,
                BinaryOperator::Divide => OpCode::Divide,
                BinaryOperator::Modulo => OpCode::Modulo,

                BinaryOperator::Equal => OpCode::Equal,
                BinaryOperator::NotEqual => OpCode::NotEqual,
                BinaryOperator::LessThan => OpCode::LessThan,
                BinaryOperator::GreaterThan => OpCode::GreaterThan,
                BinaryOperator::LessThanOrEqual => OpCode::LessThanOrEqual,
                BinaryOperator::GreaterThanOrEqual => OpCode::GreaterThanOrEqual,

                BinaryOperator::And => OpCode::And,
                BinaryOperator::Or => OpCode::Or,
            });
        }

        Expression::Unary { operator, operand } => {
            compile_expression(operand, instructions, unresolved_calls)?;

            instructions.push(match operator {
                UnaryOperator::Negate => OpCode::Negate,
                UnaryOperator::Not => OpCode::Not,
            });
        }

        Expression::Match { value, arms } => {
            compile_match_expression(value, arms, instructions, unresolved_calls)?;
        }

        Expression::Array(elements) => {
            for element in elements {
                compile_expression(element, instructions, unresolved_calls)?;
            }

            instructions.push(OpCode::CreateArray(elements.len()));
        }
    }

    Ok(())
}
