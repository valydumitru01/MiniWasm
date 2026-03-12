use crate::backend::x86_64_emitter::X86_64Emitter;
use crate::runtime::module::CompiledFunction;
use crate::wasm::types::{Instruction, WasmFunction, WasmModule};

pub trait Compiler {
    fn name(&self) -> &str;
    fn compile_module(&self, module: &WasmModule, import_pointers: &[*const u8], memory_ptr: *const u8) -> anyhow::Result<Vec<CompiledFunction>>;
}

struct X86_64Compiler;
impl Compiler for X86_64Compiler {
    fn name(&self) -> &str {
        todo!()
    }

    fn compile_module(&self, module: &WasmModule, import_pointers: &[*const u8], memory_ptr: *const u8) -> anyhow::Result<Vec<CompiledFunction>> {
        todo!()
    }


    // For each WasmFunction:
    //   1. Emit prologue
    //   2. Walk instructions, emit x86_64 (singlepass, like Wasmer)
    //   3. Emit epilogue
    //   4. Return CompiledFunction
}


impl X86_64Compiler {
    fn local_offset(local_idx: u32) -> i8 {
        // Locals are at RBP - 16 (return addr + old RBP) - 8 * local_idx
        -16 - (local_idx as i8 * 8)
    }
    fn compile_function(&self, func: &WasmFunction) -> CompiledFunction {
        let mut emit = X86_64Emitter::new();
        let frame_size = 16 + func.locals.len() as u8 * 8;  // 16 for return address + old RBP, 8 per local
        // Prologue
        emit.push_rbp();
        emit.mov_rbp_rsp();
        emit.sub_rsp_imm8(frame_size);    // space for locals

        // Singlepass: one instruction at a time
        for instr in &func.body {
            match instr {
                Instruction::I32Const(val) => {
                    emit.mov_eax_imm32(*val);
                    emit.push_rax();
                }
                Instruction::LocalGet(idx) => {
                    emit.mov_rax_rbp_disp8(Self::local_offset(*idx));
                    emit.push_rax();
                }
                Instruction::LocalSet(idx) => {
                    emit.pop_rax();
                    emit.mov_rbp_disp8_rax(Self::local_offset(*idx));
                }
                Instruction::I32Add => {
                    emit.pop_rcx();     // right
                    emit.pop_rax();     // left
                    emit.add_rax_rcx();
                    emit.push_rax();    // result
                }
                Instruction::Call(func_idx) => {
                    // Pop args, put in RCX/RDX/R8/R9 (Windows x64)
                    // Allocate shadow space
                    // Load function pointer, call
                }
                Instruction::Return => {
                    emit.pop_rax();     // return value
                    emit.mov_rsp_rbp();
                    emit.pop_rbp();
                    emit.ret();
                }
                // ... other instructions
                Instruction::F64Const(_) => {}
                Instruction::I32Sub => {}
                Instruction::I32Mul => {}
                Instruction::I32Div => {}
                Instruction::F64Add => {}
                Instruction::F64Sub => {}
                Instruction::F64Mul => {}
                Instruction::F64Div => {}
                Instruction::I32Load(_) => {}
                Instruction::I32Store(_) => {}
                Instruction::MemoryGrow => {}
            }
        }

        // Epilogue (implicit return 0 if no explicit return)
        emit.mov_rsp_rbp();
        emit.pop_rbp();
        emit.ret();

        CompiledFunction { code: emit.into_bytes(), func_type: func.func_type.clone() }
    }
}