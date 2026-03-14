//! Bytecode CEK virtual machine.
//!
//! This module is a work-in-progress prototype. The VM compiles UPLC
//! terms to a flat bytecode array and executes them with a tight
//! dispatch loop, eliminating AST pointer chasing.
//!
//! TODO:
//! - Proper Value::LambdaBC / Value::DelayBC variants instead of
//!   abusing DeBruijn parameter for body_ip storage
//! - Integrate builtin call/costing directly instead of delegating
//!   to Machine
//! - Full error reporting (currently uses ExplicitErrorTerm for all errors)
//! - Benchmarking and optimization

// Placeholder — full implementation in progress.
// The compiler (compiler.rs) and opcode definitions (mod.rs) are ready.
// The VM execution loop needs the Value type extended with bytecode
// closure variants before it can be completed correctly.
