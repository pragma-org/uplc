# Bytecode VM Design for UPLC CEK Machine

## Architecture Overview

Compile the AST (`Term` tree) into a flat `Vec<u8>` bytecode buffer, plus a side-table of constants/pointers, and run a tight dispatch loop. The VM retains the CEK architecture: environment (variable bindings), value stack (implicit in continuations), and continuation stack (pending work). Instead of pattern-matching on `Term` enum variants and chasing pointers through the arena, the instruction pointer walks linearly through a byte array.

## 1. Bytecode Encoding

**Layout**: Single `Vec<u8>` bytecode buffer. Opcodes are 1 byte. Operands inline after opcode.

**Operand encoding**:
- Small integers (de Bruijn indices, tags, branch counts): `u8` (0-255)
- Offsets (jump targets): `u32` little-endian (absolute positions in bytecode)
- Constant references: `u16` index into constant pool

## 2. Opcode Table

### Core Opcodes

| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| VAR | 0x01 | index: u8 | Lookup env[index], return value |
| LAMBDA | 0x02 | body_offset: u32 | Create closure(ip=body_offset, env), return |
| APPLY | 0x03 | arg_offset: u32 | Push FrameAwaitFunTerm, compute function |
| DELAY | 0x04 | body_offset: u32 | Create thunk(ip=body_offset, env), return |
| FORCE | 0x05 | (none) | Push FrameForce, compute inner |
| CONSTR | 0x06 | tag: u8, nfields: u8, offsets: [u32; n] | Constructor |
| CASE | 0x07 | nbranches: u8, offsets: [u32; n] | Pattern match |
| CONST | 0x08 | index: u16 | Load constant_pool[index], return |
| BUILTIN | 0x09 | fun_id: u8 | Create Runtime(fun_id), return |
| ERROR | 0x0A | (none) | Halt with error |

### Superinstructions

| Opcode | Byte | Operands | Replaces |
|--------|------|----------|----------|
| FORCE_DELAY | 0x10 | (none) | Force(Delay(body)) → compute body inline |
| APPLY_LAMBDA | 0x11 | body_offset: u32 | Apply(Lambda(body), arg) → push frame, compute arg |
| FORCE_BUILTIN | 0x12 | fun_id: u8 | Force(Builtin(f)) → create+force runtime |
| FORCE2_BUILTIN | 0x13 | fun_id: u8 | Force(Force(Builtin(f))) → create+force²  |

### Specialized Constants

| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| CONST_UNIT | 0x20 | (none) | Unit constant |
| CONST_TRUE | 0x21 | (none) | Boolean true |
| CONST_FALSE | 0x22 | (none) | Boolean false |
| CONST_SMALL_INT | 0x23 | value: i8 | Small integer (-128..127) |

## 3. Constants Strategy

**Constant pool**: Separate `Vec<&'a Constant<'a>>` alongside bytecode. `CONST` opcode indexes into pool by u16. Avoids encoding variable-size data (BigInts, bytestrings, PlutusData) in the bytecode stream.

Specialized constant opcodes (`CONST_UNIT`, `CONST_TRUE`, etc.) avoid pool lookup entirely for the most frequent constants.

## 4. Case/Branch Handling

```
CASE nbranches: u8
  branch_0_offset: u32
  branch_1_offset: u32
  ...
[scrutinee bytecode follows]
```

When scrutinee evaluates to `Constr(tag, fields)`:
1. Look up `branch_offsets[tag]`
2. Push fields as `FrameAwaitFunValue` frames
3. Set `ip = branch_offsets[tag]`

## 5. Apply Encoding

```
APPLY arg_offset: u32
[function bytecode]
[arg bytecode]           ← arg_offset points here
```

VM reads offset, pushes `FrameAwaitFunTerm(saved_ip=arg_offset, env)`, computes function starting at ip+5.

```
APPLY_LAMBDA body_offset: u32
[arg bytecode]
[body bytecode]          ← body_offset points here
```

## 6. Compilation Strategy

Single-pass recursive traversal with backpatching:
1. Walk AST, emit opcodes + placeholder u32 holes for forward references
2. When target is known, patch the hole with actual offset
3. Detect superinstruction patterns via nested match on Term variants

## 7. Value Type Extensions

Need new variants for bytecode closures:

```rust
Value::Lambda {
    body_ip: u32,          // bytecode offset (not Term pointer)
    env: &'a Env<'a>,
}
Value::Delay {
    body_ip: u32,
    env: &'a Env<'a>,
}
```

The `parameter` field in Lambda is unused at runtime for de Bruijn — only needed for discharge (error reporting).

## 8. Performance Wins

1. **Cache locality**: Contiguous `Vec<u8>` vs scattered arena Term nodes
2. **Smaller working set**: 1-byte opcodes vs 16-24 byte Term enum + pointer indirection
3. **Branch prediction**: Jump table on ~20 opcodes vs larger enum discriminant
4. **Superinstructions**: Eliminate continuation frame push/pop for common patterns
5. **Constant pool**: Pre-interned, no per-lookup wrapping

## 9. Implementation Phases

1. ✅ **Phase 1**: Opcode definitions + compiler + compiler tests
2. 🔧 **Phase 2**: Extend Value with bytecode variants, implement VM dispatch loop
3. **Phase 3**: Conformance tests passing via bytecode path
4. **Phase 4**: Benchmark, superinstruction tuning, profile-guided optimization
