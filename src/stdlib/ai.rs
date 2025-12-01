// src/stdlib/ai.rs
use crate::runtime::{AVM, Value};
// use std::env::args;

pub fn register_ai_primitives(vm: &mut AVM) {
    vm.register_native("load_model", load_model);
    vm.register_native("predict", predict);
}

fn load_model(args: Vec<Value>) -> Value {
    println!("Loading model {:?}", args[0]);
    Value::Str("model_handle_1".to_string())
}

fn predict(args: Vec<Value>) -> Value {
    println!("Running inference with input {:?}", args[1]);
    Value::Str("predicted_output".to_string())
}
