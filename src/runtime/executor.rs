// src/runtime/executor.rs

use std::collections::HashMap;

use crate::runtime::{AVM, OpCode, Value};

/// A single active function invocation.
struct CallFrame {
    return_ip: usize,
    stack_base: usize,

    // The first scope contains function parameters and top-level
    // function variables. Additional scopes represent nested blocks.
    scopes: Vec<HashMap<String, Value>>,
}

const MAX_CALL_DEPTH: usize = 1024;
const DEFAULT_MAX_INSTRUCTIONS: u64 = 1_000_000;

/// Execute AVM instructions using the default instruction limit.
pub fn execute_with_output(vm: &mut AVM) -> Result<String, String> {
    execute_with_output_and_limit(vm, DEFAULT_MAX_INSTRUCTIONS)
}

/// Execute AVM instructions with a configurable instruction limit.
///
/// The limit counts every executed AVM instruction, including
/// instructions executed repeatedly inside loops and functions.
pub fn execute_with_output_and_limit(
    vm: &mut AVM,
    max_instructions: u64,
) -> Result<String, String> {
    let mut executed_instructions: u64 = 0;
    let mut ip = 0;
    let mut output = String::new();
    let mut call_stack: Vec<CallFrame> = Vec::new();

    while ip < vm.instructions.len() {
        if executed_instructions >= max_instructions {
            return Err(format!(
                "Maximum instruction limit ({}) exceeded at ip={}",
                max_instructions, ip
            ));
        }

        executed_instructions += 1;

        let opcode = vm.instructions[ip].opcode.clone();

        // Default behaviour: advance to the next instruction.
        // Jump, Call and Return can override this.
        let mut next_ip = ip + 1;

        match opcode {
            OpCode::Nop => {}

            // -------------------------------------------------
            // Values
            // -------------------------------------------------
            OpCode::PushInt(value) => {
                vm.stack.push(Value::Int(value));
            }

            OpCode::PushFloat(value) => {
                vm.stack.push(Value::Float(value));
            }

            OpCode::PushBool(value) => {
                vm.stack.push(Value::Bool(value));
            }

            OpCode::PushChar(value) => {
                vm.stack.push(Value::Char(value));
            }

            OpCode::PushString(value) => {
                vm.stack.push(Value::Str(value));
            }

            OpCode::PushNull => {
                vm.stack.push(Value::Null);
            }

            OpCode::BuildString(count) => {
                if vm.stack.len() < count {
                    return Err(format!(
                        "Stack underflow on BuildString: expected {} values, found {} at ip={}",
                        count,
                        vm.stack.len(),
                        ip
                    ));
                }

                let mut segments = Vec::with_capacity(count);

                for _ in 0..count {
                    segments.push(pop_value(&mut vm.stack, "BuildString", ip)?);
                }

                segments.reverse();

                let mut result = String::new();

                for segment in &segments {
                    result.push_str(&format_value(segment));
                }

                vm.stack.push(Value::Str(result));
            }

            OpCode::CreateArray(count) => {
                if vm.stack.len() < count {
                    return Err(format!(
                        "Stack underflow on CreateArray: expected {} values, found {} at ip={}",
                        count,
                        vm.stack.len(),
                        ip
                    ));
                }

                let mut elements = Vec::with_capacity(count);

                for _ in 0..count {
                    elements.push(pop_value(&mut vm.stack, "CreateArray", ip)?);
                }

                // Values are popped in reverse order.
                // Restore their original source order.
                elements.reverse();

                // Enforce homogeneous element types at runtime as well.
                if let Some(first) = elements.first() {
                    for element in elements.iter().skip(1) {
                        if !same_value_type(first, element) {
                            return Err(format!(
                                "Heterogeneous array elements at ip={}: {:?} and {:?}",
                                ip, first, element
                            ));
                        }
                    }
                }

                vm.stack.push(Value::Array(elements));
            }

            OpCode::ArrayLength => {
                let value = pop_value(&mut vm.stack, "ArrayLength", ip)?;

                match value {
                    Value::Array(elements) => {
                        let length = i64::try_from(elements.len()).map_err(|_| {
                            format!("Array length exceeds supported integer range at ip={}", ip)
                        })?;

                        vm.stack.push(Value::Int(length));
                    }

                    other => {
                        return Err(format!(
                            "ArrayLength requires an array at ip={}, found {:?}",
                            ip, other
                        ));
                    }
                }
            }

            OpCode::ArrayGet => {
                // Stack order: array, index.
                let index_value = pop_value(&mut vm.stack, "ArrayGet", ip)?;

                let array_value = pop_value(&mut vm.stack, "ArrayGet", ip)?;

                let index = match index_value {
                    Value::Int(index) if index >= 0 => usize::try_from(index)
                        .map_err(|_| format!("Array index is too large at ip={}", ip))?,

                    Value::Int(index) => {
                        return Err(format!(
                            "Array index cannot be negative at ip={}: {}",
                            ip, index
                        ));
                    }

                    other => {
                        return Err(format!(
                            "Array index must be an integer at ip={}, found {:?}",
                            ip, other
                        ));
                    }
                };

                match array_value {
                    Value::Array(elements) => {
                        let element = elements.get(index).cloned().ok_or_else(|| {
                            format!(
                                "Array index out of bounds at ip={}: index {}, length {}",
                                ip,
                                index,
                                elements.len()
                            )
                        })?;

                        vm.stack.push(element);
                    }

                    other => {
                        return Err(format!(
                            "ArrayGet requires an array at ip={}, found {:?}",
                            ip, other
                        ));
                    }
                }
            }

            // -------------------------------------------------
            // Variables
            // -------------------------------------------------
            OpCode::DefineVar(name) => {
                let value = pop_value(&mut vm.stack, "DefineVar", ip)?;

                if let Some(frame) = call_stack.last_mut() {
                    let current_scope = frame
                        .scopes
                        .last_mut()
                        .ok_or_else(|| format!("Function has no active scope at ip={}", ip))?;

                    if current_scope.contains_key(&name) {
                        return Err(format!("Duplicate local variable '{}' at ip={}", name, ip));
                    }

                    current_scope.insert(name, value);
                } else if let Some(current_scope) = vm.scopes.last_mut() {
                    if current_scope.contains_key(&name) {
                        return Err(format!("Duplicate scoped variable '{}' at ip={}", name, ip));
                    }

                    current_scope.insert(name, value);
                } else {
                    if vm.variables.contains_key(&name) {
                        return Err(format!("Duplicate global variable '{}' at ip={}", name, ip));
                    }

                    vm.variables.insert(name, value);
                }
            }

            OpCode::StoreVar(name) => {
                let value = pop_value(&mut vm.stack, "StoreVar", ip)?;

                let mut stored = false;

                if let Some(frame) = call_stack.last_mut() {
                    // Search function-local scopes first.
                    for scope in frame.scopes.iter_mut().rev() {
                        if let Some(existing) = scope.get_mut(&name) {
                            *existing = value.clone();
                            stored = true;
                            break;
                        }
                    }

                    // Then search global variables.
                    if !stored {
                        if let Some(existing) = vm.variables.get_mut(&name) {
                            *existing = value.clone();
                            stored = true;
                        }
                    }

                    // If no variable exists, create it in the current function scope.
                    if !stored {
                        let current_scope = frame
                            .scopes
                            .last_mut()
                            .ok_or_else(|| format!("Function has no active scope at ip={}", ip))?;

                        current_scope.insert(name, value);
                    }
                } else {
                    // Main execution: search active global lexical scopes.
                    for scope in vm.scopes.iter_mut().rev() {
                        if let Some(existing) = scope.get_mut(&name) {
                            *existing = value.clone();
                            stored = true;
                            break;
                        }
                    }

                    // Then search global variables.
                    if !stored {
                        if let Some(existing) = vm.variables.get_mut(&name) {
                            *existing = value.clone();
                            stored = true;
                        }
                    }

                    // Otherwise, create the variable in the current global scope.
                    if !stored {
                        if let Some(current_scope) = vm.scopes.last_mut() {
                            current_scope.insert(name, value);
                        } else {
                            vm.variables.insert(name, value);
                        }
                    }
                }
            }

            OpCode::LoadVar(name) => {
                let value = if let Some(frame) = call_stack.last() {
                    frame
                        .scopes
                        .iter()
                        .rev()
                        .find_map(|scope| scope.get(&name).cloned())
                } else {
                    vm.scopes
                        .iter()
                        .rev()
                        .find_map(|scope| scope.get(&name).cloned())
                }
                .or_else(|| vm.variables.get(&name).cloned())
                .ok_or_else(|| format!("Undefined variable '{}' at ip={}", name, ip))?;

                vm.stack.push(value);
            }
            // -------------------------------------------------
            // Control flow
            // -------------------------------------------------
            OpCode::EnterScope => {
                if let Some(frame) = call_stack.last_mut() {
                    frame.scopes.push(HashMap::new());
                } else {
                    vm.scopes.push(HashMap::new());
                }
            }

            OpCode::ExitScope => {
                if let Some(frame) = call_stack.last_mut() {
                    if frame.scopes.len() <= 1 {
                        return Err(format!(
                            "Cannot exit the function's base scope at ip={}",
                            ip
                        ));
                    }

                    frame.scopes.pop();
                } else {
                    vm.scopes
                        .pop()
                        .ok_or_else(|| format!("No active global scope to exit at ip={}", ip))?;
                }
            }

            OpCode::Jump(target) => {
                if target > vm.instructions.len() {
                    return Err(format!("Invalid jump target {} at ip={}", target, ip));
                }

                next_ip = target;
            }

            OpCode::JumpIfFalse(target) => {
                // Validate the destination before changing execution flow.
                if target > vm.instructions.len() {
                    return Err(format!(
                        "Invalid conditional jump target {} at ip={}",
                        target, ip
                    ));
                }

                // Consume the condition from the operand stack.
                let condition = pop_value(&mut vm.stack, "JumpIfFalse", ip)?;

                match condition {
                    Value::Bool(false) => {
                        next_ip = target;
                    }

                    Value::Bool(true) => {
                        // Continue with the next instruction.
                    }

                    other => {
                        return Err(format!(
                            "JumpIfFalse requires a Boolean condition at ip={}, found {:?}",
                            ip, other
                        ));
                    }
                }
            }

            // -------------------------------------------------
            // Function calls
            // -------------------------------------------------
            OpCode::Call {
                target,
                argument_count,
            } => {
                if target >= vm.instructions.len() {
                    return Err(format!(
                        "Invalid function entry point {} at ip={}",
                        target, ip
                    ));
                }

                if vm.stack.len() < argument_count {
                    return Err(format!(
                        "Stack underflow on Call: expected {} arguments, found {} at ip={}",
                        argument_count,
                        vm.stack.len(),
                        ip
                    ));
                }

                let stack_base = vm.stack.len() - argument_count;

                if call_stack.len() >= MAX_CALL_DEPTH {
                    return Err(format!(
                        "Maximum function call depth ({}) exceeded at ip={}",
                        MAX_CALL_DEPTH, ip
                    ));
                }

                call_stack.push(CallFrame {
                    return_ip: ip + 1,
                    stack_base,
                    scopes: vec![HashMap::new()],
                });

                // The function prologue binds its arguments.
                // It executes at the target instruction.
                next_ip = target;
            }

            OpCode::Return => {
                let frame = call_stack
                    .pop()
                    .ok_or_else(|| format!("Return outside a function at ip={}", ip))?;

                if vm.stack.len() <= frame.stack_base {
                    return Err(format!("Function returned without a result at ip={}", ip));
                }

                let return_value = pop_value(&mut vm.stack, "Return", ip)?;

                // Remove any remaining temporary values belonging
                // to this invocation.
                vm.stack.truncate(frame.stack_base);

                // Place the result where the caller expects it.
                vm.stack.push(return_value);

                next_ip = frame.return_ip;
            }

            OpCode::Pop => {
                pop_value(&mut vm.stack, "Pop", ip)?;
            }

            // -------------------------------------------------
            // Arithmetic
            // -------------------------------------------------
            OpCode::Add => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "Add", ip)?;
                vm.stack.push(arithmetic(a, b, Arithmetic::Add, ip)?);
            }

            OpCode::Subtract => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "Subtract", ip)?;
                vm.stack.push(arithmetic(a, b, Arithmetic::Subtract, ip)?);
            }

            OpCode::Multiply => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "Multiply", ip)?;
                vm.stack.push(arithmetic(a, b, Arithmetic::Multiply, ip)?);
            }

            OpCode::Divide => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "Divide", ip)?;
                vm.stack.push(arithmetic(a, b, Arithmetic::Divide, ip)?);
            }

            OpCode::Modulo => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "Modulo", ip)?;
                vm.stack.push(arithmetic(a, b, Arithmetic::Modulo, ip)?);
            }

            // -------------------------------------------------
            // Comparisons
            // -------------------------------------------------
            OpCode::Equal => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "Equal", ip)?;
                vm.stack.push(Value::Bool(values_equal(&a, &b)?));
            }

            OpCode::NotEqual => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "NotEqual", ip)?;
                vm.stack.push(Value::Bool(!values_equal(&a, &b)?));
            }

            OpCode::LessThan => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "LessThan", ip)?;

                vm.stack
                    .push(Value::Bool(compare_values(&a, &b, Comparison::Less, ip)?));
            }

            OpCode::GreaterThan => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "GreaterThan", ip)?;

                vm.stack.push(Value::Bool(compare_values(
                    &a,
                    &b,
                    Comparison::Greater,
                    ip,
                )?));
            }

            OpCode::LessThanOrEqual => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "LessThanOrEqual", ip)?;

                vm.stack.push(Value::Bool(compare_values(
                    &a,
                    &b,
                    Comparison::LessOrEqual,
                    ip,
                )?));
            }

            OpCode::GreaterThanOrEqual => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "GreaterThanOrEqual", ip)?;

                vm.stack.push(Value::Bool(compare_values(
                    &a,
                    &b,
                    Comparison::GreaterOrEqual,
                    ip,
                )?));
            }

            // -------------------------------------------------
            // Logical operations
            // -------------------------------------------------
            OpCode::And => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "And", ip)?;

                match (a, b) {
                    (Value::Bool(a), Value::Bool(b)) => {
                        vm.stack.push(Value::Bool(a && b));
                    }

                    (a, b) => {
                        return Err(format!("Type error on And at ip={}: {:?}, {:?}", ip, a, b));
                    }
                }
            }

            OpCode::Or => {
                let (a, b) = pop_binary_operands(&mut vm.stack, "Or", ip)?;

                match (a, b) {
                    (Value::Bool(a), Value::Bool(b)) => {
                        vm.stack.push(Value::Bool(a || b));
                    }

                    (a, b) => {
                        return Err(format!("Type error on Or at ip={}: {:?}, {:?}", ip, a, b));
                    }
                }
            }

            // -------------------------------------------------
            // Unary operations
            // -------------------------------------------------
            OpCode::Negate => {
                let value = pop_value(&mut vm.stack, "Negate", ip)?;

                let result = match value {
                    Value::Int(value) => Value::Int(
                        value
                            .checked_neg()
                            .ok_or_else(|| format!("Integer overflow on Negate at ip={}", ip))?,
                    ),

                    Value::Float(value) => Value::Float(-value),

                    other => {
                        return Err(format!(
                            "Invalid operand for Negate at ip={}: {:?}",
                            ip, other
                        ));
                    }
                };

                vm.stack.push(result);
            }

            OpCode::Not => {
                let value = pop_value(&mut vm.stack, "Not", ip)?;

                match value {
                    Value::Bool(value) => {
                        vm.stack.push(Value::Bool(!value));
                    }

                    other => {
                        return Err(format!("Invalid operand for Not at ip={}: {:?}", ip, other));
                    }
                }
            }

            // -------------------------------------------------
            // Output
            // -------------------------------------------------
            OpCode::Print => {
                let value = pop_value(&mut vm.stack, "Print", ip)?;

                output.push_str(&format_value(&value));
                output.push('\n');
            }
        }

        ip = next_ip;
    }

    if !call_stack.is_empty() {
        return Err("Execution ended with unfinished function calls".to_string());
    }

    Ok(output)
}

/// Preserve the original console-oriented execute interface.
pub fn execute(vm: &mut AVM, _bytecode: Vec<u8>) -> Result<(), String> {
    let output = execute_with_output(vm)?;
    print!("{}", output);
    Ok(())
}

// -------------------------------------------------------------
// Stack helpers
// -------------------------------------------------------------

fn pop_value(stack: &mut Vec<Value>, operation: &str, ip: usize) -> Result<Value, String> {
    stack
        .pop()
        .ok_or_else(|| format!("Stack underflow on {} at ip={}", operation, ip))
}

fn pop_binary_operands(
    stack: &mut Vec<Value>,
    operation: &str,
    ip: usize,
) -> Result<(Value, Value), String> {
    if stack.len() < 2 {
        return Err(format!("Stack underflow on {} at ip={}", operation, ip));
    }

    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();

    Ok((left, right))
}

// -------------------------------------------------------------
// Arithmetic
// -------------------------------------------------------------

#[derive(Clone, Copy)]
enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

fn arithmetic(a: Value, b: Value, operation: Arithmetic, ip: usize) -> Result<Value, String> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => {
            let result = match operation {
                Arithmetic::Add => a.checked_add(b),
                Arithmetic::Subtract => a.checked_sub(b),
                Arithmetic::Multiply => a.checked_mul(b),

                Arithmetic::Divide => {
                    if b == 0 {
                        return Err(format!("Division by zero at ip={}", ip));
                    }

                    a.checked_div(b)
                }

                Arithmetic::Modulo => {
                    if b == 0 {
                        return Err(format!("Modulo by zero at ip={}", ip));
                    }

                    a.checked_rem(b)
                }
            };

            result
                .map(Value::Int)
                .ok_or_else(|| format!("Integer arithmetic error at ip={}", ip))
        }

        (Value::Float(a), Value::Float(b)) => {
            let result = match operation {
                Arithmetic::Add => a + b,
                Arithmetic::Subtract => a - b,
                Arithmetic::Multiply => a * b,

                Arithmetic::Divide => {
                    if b == 0.0 {
                        return Err(format!("Division by zero at ip={}", ip));
                    }

                    a / b
                }

                Arithmetic::Modulo => {
                    if b == 0.0 {
                        return Err(format!("Modulo by zero at ip={}", ip));
                    }

                    a % b
                }
            };

            if !result.is_finite() {
                return Err(format!("Non-finite floating-point result at ip={}", ip));
            }

            Ok(Value::Float(result))
        }

        (Value::Int(a), Value::Float(b)) => {
            arithmetic(Value::Float(a as f64), Value::Float(b), operation, ip)
        }

        (Value::Float(a), Value::Int(b)) => {
            arithmetic(Value::Float(a), Value::Float(b as f64), operation, ip)
        }

        (a, b) => Err(format!(
            "Invalid arithmetic operands at ip={}: {:?}, {:?}",
            ip, a, b
        )),
    }
}

// -------------------------------------------------------------
// Comparisons
// -------------------------------------------------------------

#[derive(Clone, Copy)]
enum Comparison {
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

fn compare_values(a: &Value, b: &Value, operation: Comparison, ip: usize) -> Result<bool, String> {
    let (a, b) = match (a, b) {
        (Value::Int(a), Value::Int(b)) => (*a as f64, *b as f64),
        (Value::Float(a), Value::Float(b)) => (*a, *b),
        (Value::Int(a), Value::Float(b)) => (*a as f64, *b),
        (Value::Float(a), Value::Int(b)) => (*a, *b as f64),

        _ => {
            return Err(format!(
                "Invalid comparison operands at ip={}: {:?}, {:?}",
                ip, a, b
            ));
        }
    };

    Ok(match operation {
        Comparison::Less => a < b,
        Comparison::Greater => a > b,
        Comparison::LessOrEqual => a <= b,
        Comparison::GreaterOrEqual => a >= b,
    })
}

fn values_equal(a: &Value, b: &Value) -> Result<bool, String> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(a == b),
        (Value::Float(a), Value::Float(b)) => Ok(a == b),
        (Value::Int(a), Value::Float(b)) => Ok((*a as f64) == *b),
        (Value::Float(a), Value::Int(b)) => Ok(*a == (*b as f64)),
        (Value::Bool(a), Value::Bool(b)) => Ok(a == b),
        (Value::Char(a), Value::Char(b)) => Ok(a == b),
        (Value::Str(a), Value::Str(b)) => Ok(a == b),
        (Value::Null, Value::Null) => Ok(true),

        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Ok(false);
            }

            for (left, right) in a.iter().zip(b.iter()) {
                if !values_equal(left, right)? {
                    return Ok(false);
                }
            }

            Ok(true)
        }

        _ => Err(format!(
            "Cannot compare incompatible values: {:?} and {:?}",
            a, b
        )),
    }
}

/// Determine whether two runtime values have the same type.
///
/// Arrays are considered to have the same type when their element
/// types match, regardless of their lengths.
fn same_value_type(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(_), Value::Int(_)) => true,
        (Value::Float(_), Value::Float(_)) => true,
        (Value::Bool(_), Value::Bool(_)) => true,
        (Value::Char(_), Value::Char(_)) => true,
        (Value::Str(_), Value::Str(_)) => true,
        (Value::Null, Value::Null) => true,

        (Value::Array(a), Value::Array(b)) => {
            match (a.first(), b.first()) {
                (Some(a_first), Some(b_first)) => same_value_type(a_first, b_first),

                (None, None) => true,

                // An empty array has no runtime element type to compare.
                _ => false,
            }
        }

        _ => false,
    }
}

// -------------------------------------------------------------
// Value formatting
// -------------------------------------------------------------

fn format_value(value: &Value) -> String {
    match value {
        Value::Int(value) => value.to_string(),

        Value::Float(value) => value.to_string(),

        Value::Bool(value) => value.to_string(),

        Value::Char(value) => value.to_string(),

        Value::Str(value) => value.clone(),

        Value::Array(elements) => {
            let formatted: Vec<String> = elements.iter().map(format_value).collect();

            format!("[{}]", formatted.join(", "))
        }

        Value::Null => "null".to_string(),
    }
}
