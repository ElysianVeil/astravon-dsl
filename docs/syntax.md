# Astravon DSL Syntax Guide

## Variables
let x = 42;
let message = "Hello {name}";



## Functions
fn add(a: Int, b: Int) -> Int {
return a + b;
}



## Interpolation
Strings can interpolate variables using `{var}`:
print("Hello {name}");