// src/stdlib/bio.rs
use crate::runtime::{AVM, Value};
// use std::env::args;

pub fn register_bio_primitives(vm: &mut AVM) {
    vm.register_native("encode_dna", encode_dna);
    vm.register_native("mutate_dna", mutate_dna);
}

fn encode_dna(args: Vec<Value>) -> Value {
    // args: [sequence string]
    if let Value::Str(seq) = &args[0] {
        let encoded = seq.bytes().map(|b| b ^ 0b1010).collect::<Vec<u8>>();
        Value::Str(base64::encode(encoded))
    } else {
        Value::Str("error".to_string())
    }
}

fn mutate_dna(args: Vec<Value>) -> Value {
    // args: [encoded_sequence, mutation_rate]
    // stub
    Value::Str("mutated_sequence".to_string())
}
