use crate::wasm::types::{FuncType, WasmModule};

pub struct CompiledFunction {
    pub code: Vec<u8>,        // raw x86_64 bytes
    pub func_type: FuncType,
}

pub struct CompiledModule {
    pub wasm: WasmModule,                    // original IR (for linking)
    pub functions: Vec<CompiledFunction>,    // compiled native code
    pub memory_pages: u32,                    // for memory allocation
}

impl CompiledModule {
    pub fn new(wasm: WasmModule, functions: Vec<CompiledFunction>) -> Self {
        let memory_pages = wasm.memory_pages;
        Self {
            wasm,
            functions,
            memory_pages,
        }
    }
}