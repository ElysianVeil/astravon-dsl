// src/stdlib/realm.rs
use crate::runtime::{AVM, Value};
// use std::env::args;

pub fn register_realm_primitives(vm: &mut AVM) {
    vm.register_native("spawn_entity", spawn_entity);
    vm.register_native("advance_tick", advance_tick);
}

fn spawn_entity(args: Vec<Value>) -> Value {
    println!("Spawning entity {:?}", args[0]);
    Value::Str("entity_id_42".to_string())
}

fn advance_tick(args: Vec<Value>) -> Value {
    println!("Advancing simulation tick");
    Value::Int(1)
}
