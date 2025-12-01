# Astravon Virtual Machine Specification

The AVM executes Astravon bytecode securely in sandboxed environments.

- **Instruction Set**: Load, Store, Add, Call, ChainCall…
- **Memory Model**: Linear segmented memory with GC.
- **Security**: Capability-based permissions for blockchain/biotech calls.
- **Targets**: LLVM IR, WebAssembly, or native AVM bytecode.

See `/src/runtime/` for the Rust implementation.
