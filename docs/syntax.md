# AstravonDSL Syntax Guide

**Project:** House of Astravon  
**Component:** AstravonDSL  
**Document:** Language Syntax Reference  
**Version:** 0.2  
**Status:** Initial syntax specification

---

# 1. Introduction

AstravonDSL uses a minimal, readable syntax influenced by Python and Rust.

The language is designed to support general programming constructs alongside specialized domain capabilities for AI, blockchain, biotechnology, simulation, and robotics.

This document defines the intended source syntax.

Examples marked as planned illustrate the language's design direction and do not necessarily represent functionality supported by the current compiler.

# 2. Basic Program Structure

An AstravonDSL program consists of statements and expressions.

Example:

```avon
print "Hello Astravon!"
print 42
print true
```

Statements are processed in source order.

Semicolons may be used as statement terminators in the proposed syntax. Newline-based statement termination should also be supported where the grammar can identify statement boundaries unambiguously.

# 3. Variables

Variables are declared using `let`.

Syntax:

```avon
let identifier = expression;
```

Examples:

```avon
let x = 42;
let message = "Hello Astravon";
let temperature = 36.5;
let active = true;
let initial = 'A';
let result = null;
```

The compiler should infer a variable's type from its initial expression where possible.

Explicit type annotations are also supported by the language design.

Example:

```avon
let age: Int = 20;
let name: String = "Astravon";
let active: Bool = true;
```

The initial type system includes:

| Type | Example |
|---|---|
| `Int` | `42` |
| `Float` | `3.14` |
| `Bool` | `true` |
| `Char` | `'A'` |
| `String` | `"Astravon"` |
| `Null` | `null` |

Variable mutability, reassignment, and scope rules must be defined by the language's semantic specification.

# 4. Data Types and Literals

## 4.1 Integers

Signed whole numbers.

```avon
let age = 20;
let balance = 1500;
print age;
```

## 4.2 Floating-Point Numbers

Numbers containing a decimal component.

```avon
let pi = 3.14159;
let temperature = 36.5;
print pi;
```

## 4.3 Booleans

Logical values.

```avon
let active = true;
let completed = false;

print active;
```

## 4.4 Characters

Single Unicode scalar values enclosed in single quotation marks.

```avon
let initial = 'A';
print initial;
```

## 4.5 Strings

Text enclosed in double quotation marks.

```avon
let name = "Astravon";
let message = "Welcome to AstravonDSL";
print message;
```

## 4.6 Null

Represents the absence of a value.

```avon
let result = null;
print result;
```

# 5. Output

The `print` statement outputs an expression.

Syntax:

```avon
print expression;
```

Examples:

```avon
print "Hello Astravon!";
print 42;
print 3.14;
print true;
print 'A';
print null;
```

The expression may be a literal, variable, arithmetic expression, or function result.

# 6. String Interpolation

Strings support interpolation using braces.

Syntax:

```avon
"Text {expression}"
```

Example:

```avon
let name = "Astravon";
let message = "Hello {name}";

print message;
```

Expected output:

```text
Hello Astravon
```

Interpolation should support expressions as the language evolves.

Example:

```avon
let age = 20;
print "Age: {age}";
```

The compiler must distinguish interpolation expressions from ordinary string characters.

Unterminated or invalid interpolation expressions should produce clear compilation errors.

# 7. Arithmetic Expressions

AstravonDSL supports the following intended arithmetic operators:

| Operator | Meaning | Example |
|---|---|---|
| `+` | Addition | `a + b` |
| `-` | Subtraction | `a - b` |
| `*` | Multiplication | `a * b` |
| `/` | Division | `a / b` |
| `%` | Modulo | `a % b` |

Example:

```avon
let a = 20;
let b = 22;

let result = a + b;

print result;
```

Operator precedence and numeric conversion rules must be defined consistently by the parser and semantic analyzer.

# 8. Comparison and Logical Expressions

## 8.1 Comparison Operators

| Operator | Meaning |
|---|---|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

Example:

```avon
let age = 20;
let eligible = age >= 18;

print eligible;
```

## 8.2 Logical Operators

The intended logical operators are:

| Operator | Meaning |
|---|---|
| `&&` | Logical AND |
| `||` | Logical OR |
| `!` | Logical NOT |

Example:

```avon
let authenticated = true;
let authorized = false;

let permitted = authenticated && authorized;

print permitted;
```

# 9. Functions

Functions are declared using `fn`.

Syntax:

```avon
fn function_name(parameter: Type) -> ReturnType {
    // Function body
}
```

Example:

```avon
fn add(a: Int, b: Int) -> Int {
    return a + b;
}
```

Functions may accept multiple parameters and return values.

Function calls use parentheses:

```avon
let result = add(20, 22);

print result;
```

A function without a meaningful return value may eventually use a `Void` return type.

Function declarations, parameter validation, return-type checking, and execution frames are part of the planned language implementation.

# 10. Control Flow

The intended control-flow syntax includes conditional statements and loops.

## 10.1 If Statements

```avon
if age >= 18 {
    print "Adult";
} else {
    print "Minor";
}
```

## 10.2 While Loops

```avon
let count = 0;

while count < 5 {
    print count;
    count = count + 1;
}
```

## 10.3 For Loops

```avon
for item in collection {
    print item;
}
```

The precise iteration and range syntax will be established as collection support is implemented.

# 11. Comments

Single-line comments begin with `#`.

Example:

```avon
# This is an AstravonDSL comment
print "Hello Astravon!";
```

Comments are ignored during compilation.

# 12. Error Handling

The language should provide explicit diagnostics for:

- Invalid tokens.
- Malformed expressions.
- Undefined variables.
- Type mismatches.
- Invalid function calls.
- Incorrect return values.
- Invalid interpolation.
- Unsupported operations.

Errors should identify their source location wherever possible.

# 13. Domain-Specific Syntax

AstravonDSL is intended to support specialized language constructs for its target domains.

These constructs will be introduced through a consistent syntax and type system.

## 13.1 AI

Potential future syntax:

```avon
# Illustrative syntax; not yet implemented

let model = ai.load("model");
let result = ai.infer(model, input);
```

## 13.2 Blockchain

Potential future syntax:

```avon
# Illustrative syntax; not yet implemented

let transaction = chain.create_transaction();
let result = chain.submit(transaction);
```

## 13.3 Biotechnology

Potential future syntax:

```avon
# Illustrative syntax; not yet implemented

let sequence = dna.sequence("ATCG");
let result = dna.analyze(sequence);
```

These examples illustrate intended domain interfaces. Their APIs, types, permissions, and exact syntax remain subject to implementation.

# 14. Syntax Design Rules

AstravonDSL follows these rules:

1. Syntax should be readable and predictable.
2. Expressions should be reusable across statements.
3. Types should have clear and consistent meanings.
4. Invalid operations should be rejected.
5. Domain-specific constructs should follow common language conventions.
6. Compiler diagnostics should be understandable.
7. New syntax should preserve consistency with existing constructs.

# 15. Implementation Status

| Feature | Status |
|---|---|
| String literals | Initial implementation |
| Integer literals | Lexer/parser support being developed |
| Float literals | Lexer/parser support being developed |
| Boolean literals | Planned integration |
| Character literals | Planned integration |
| Null literal | Planned integration |
| Print statements | Initial implementation |
| Variable declarations | Planned |
| Type annotations | Planned |
| String interpolation | Planned |
| Arithmetic expressions | Initial runtime addition support; parser integration pending |
| Comparison operators | Planned |
| Logical operators | Planned |
| Function declarations | Planned |
| Function calls | Planned |
| Conditional statements | Planned |
| Loops | Planned |
| Domain-specific syntax | Planned |

# 16. Long-Term Objective

AstravonDSL aims to provide a simple but powerful programming language for the House of Astravon ecosystem.

Its syntax should remain accessible while its compiler and runtime provide the security, portability, and extensibility required for specialized computation.

**ASTRAVON**

*Innovation • Integrity • Sovereignty*