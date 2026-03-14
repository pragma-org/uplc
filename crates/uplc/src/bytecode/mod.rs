pub mod compiler;
pub mod vm;

use crate::constant::Constant;

/// Opcodes for the UPLC bytecode VM.
/// Each opcode is a single byte. Operands follow inline.
#[repr(u8)]
#[allow(dead_code)]
pub enum Op {
    /// Var index:u8 — lookup env[index], return value
    Var = 0x01,
    /// Lambda body_offset:u32 — create closure, return it
    Lambda = 0x02,
    /// Apply arg_offset:u32 — push FrameAwaitFunTerm, compute function
    Apply = 0x03,
    /// Delay body_offset:u32 — create thunk, return it
    Delay = 0x04,
    /// Force — push FrameForce, compute inner
    Force = 0x05,
    /// Constr tag:u8 nfields:u8 field_offsets:[u32; nfields] — constructor
    Constr = 0x06,
    /// Case nbranches:u8 branch_offsets:[u32; nbranches] — pattern match
    Case = 0x07,
    /// Const index:u16 — load constant_pool[index], return it
    Const = 0x08,
    /// Builtin fun_id:u8 — create Runtime, return it
    Builtin = 0x09,
    /// Error — halt with error
    Error = 0x0A,

    // === Superinstructions ===

    /// Force(Delay(body)) — charge both steps, compute body inline
    ForceDelay = 0x10,
    /// Apply(Lambda(body), arg) — push FrameAwaitArgForLambda(body_offset), compute arg inline
    ApplyLambda = 0x11,
    /// Force(Builtin(f)) — create runtime, force once, return
    ForceBuiltin = 0x12,
    /// Force(Force(Builtin(f))) — create runtime, force twice, return
    Force2Builtin = 0x13,

    // === Specialized constants ===

    /// Unit constant
    ConstUnit = 0x20,
    /// Boolean true
    ConstTrue = 0x21,
    /// Boolean false
    ConstFalse = 0x22,
    /// Small integer (-128..127), value:i8
    ConstSmallInt = 0x23,
}

/// A compiled UPLC program ready for bytecode execution.
pub struct CompiledProgram<'a> {
    pub bytecode: Vec<u8>,
    pub constant_pool: Vec<&'a Constant<'a>>,
    pub version: (usize, usize, usize),
}

/// Read a u32 from bytecode at the given offset (little-endian).
#[inline(always)]
pub fn read_u32(bytecode: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytecode[offset],
        bytecode[offset + 1],
        bytecode[offset + 2],
        bytecode[offset + 3],
    ])
}

/// Read a u16 from bytecode at the given offset (little-endian).
#[inline(always)]
pub fn read_u16(bytecode: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytecode[offset], bytecode[offset + 1]])
}

/// Write a u32 to bytecode at the given offset (little-endian).
#[inline(always)]
pub fn write_u32(bytecode: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();
    bytecode[offset] = bytes[0];
    bytecode[offset + 1] = bytes[1];
    bytecode[offset + 2] = bytes[2];
    bytecode[offset + 3] = bytes[3];
}
