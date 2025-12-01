// src/stdlib/chain.rs
use crate::runtime::{AVM, Value};
// use std::env::args;

pub fn register_chain_primitives(vm: &mut AVM) {
    // This could push native functions into a registry
    vm.register_native("send_tx", send_tx);
    vm.register_native("get_balance", get_balance);
}

fn send_tx(args: Vec<Value>) -> Value {
    // args: [to_address, amount]
    println!("Sending tx to {:?} of {:?}", args[0], args[1]);
    Value::Str("tx_hash_123".to_string())
}

fn get_balance(args: Vec<Value>) -> Value {
    // args: [address]
    println!("Querying balance for {:?}", args[0]);
    Value::Int(1000)
}
