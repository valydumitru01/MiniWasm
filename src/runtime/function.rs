use crate::wasm::types::FuncType;

pub struct HostFunction {
    func_type: FuncType,
    // Store a type-erased callable: fn pointer + optional env pointer
    pub(crate) trampoline: *const u8,  // raw fn pointer the JIT can call
}