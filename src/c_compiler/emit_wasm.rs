use crate::c_compiler::ast::{Expr, Function, Op, Program, Stmt, StructDef, Type};
use crate::wasm::types::{
    DataSection, FuncType, Instruction, ValType, WasmExport, WasmFunction, WasmImport, WasmModule,
};
use std::collections::HashMap;

/// Emits a WasmModule from a parsed C Program.
///
/// Layout conventions:
///   - Imports come first in the function index space (indices 0..N)
///   - User-defined functions follow (indices N..N+M)
///   - String literals are placed in linear memory via data sections
///   - Struct fields are accessed via I32Load/I32Store with byte offsets
pub fn emit(program: Program) -> WasmModule {
    let mut emitter = WasmEmitter::new();
    emitter.emit_program(program)
}

struct WasmEmitter {
    /// Maps import name → function index
    import_indices: HashMap<String, u32>,
    /// Maps user function name → function index
    function_indices: HashMap<String, u32>,
    /// Maps struct name → field layout (field_name → byte offset)
    struct_layouts: HashMap<String, Vec<(String, u32)>>,
    /// Accumulated data sections for string literals
    data_sections: Vec<DataSection>,
    /// Next free byte in linear memory (for string literal placement)
    data_offset: u32,
}

impl WasmEmitter {
    fn new() -> Self {
        WasmEmitter {
            import_indices: HashMap::new(),
            function_indices: HashMap::new(),
            struct_layouts: HashMap::new(),
            data_sections: Vec::new(),
            data_offset: 0,
        }
    }

    fn emit_program(&mut self, program: Program) -> WasmModule {
        // 1. Register struct layouts
        for s in &program.structs {
            self.register_struct(s);
        }

        // 2. Build imports for known builtins
        let imports = self.build_imports();

        // 3. Assign function indices (imports first, then user functions)
        let import_count = imports.len() as u32;
        for (i, func) in program.functions.iter().enumerate() {
            self.function_indices
                .insert(func.name.clone(), import_count + i as u32);
        }

        // 4. Compile each function body
        let mut functions = Vec::new();
        let mut exports = Vec::new();
        for (i, func) in program.functions.iter().enumerate() {
            let wasm_func = self.emit_function(func);
            exports.push(WasmExport {
                name: func.name.clone(),
                func_index: import_count + i as u32,
            });
            functions.push(wasm_func);
        }

        // 5. Compute memory pages needed
        let memory_pages = if self.data_offset == 0 {
            1
        } else {
            (self.data_offset / 65536) + 1
        };

        WasmModule {
            imports,
            functions,
            exports,
            data_sections: std::mem::take(&mut self.data_sections),
            memory_pages,
        }
    }

    fn register_struct(&mut self, _s: &StructDef) {
        // TODO: compute field byte offsets (each i32 field = 4 bytes)
        // store in self.struct_layouts
        todo!()
    }

    fn build_imports(&mut self) -> Vec<WasmImport> {
        // TODO: register print_i32, print_str, etc. as imports
        // assign indices in self.import_indices
        todo!()
    }

    fn emit_function(&mut self, _func: &Function) -> WasmFunction {
        // TODO:
        // 1. Build locals list from params + local declarations
        // 2. Walk func.body statements, calling emit_stmt for each
        // 3. Return WasmFunction with accumulated instructions
        todo!()
    }

    fn emit_stmt(&mut self, _stmt: &Stmt, _instrs: &mut Vec<Instruction>) {
        // TODO: match on stmt variant:
        //   Stmt::Return(expr)         → emit_expr(expr) + Instruction::Return
        //   Stmt::Let(name, ty, init)  → allocate local slot, emit init if Some
        //   Stmt::Assign(name, expr)   → emit_expr(expr) + LocalSet(slot)
        //   Stmt::FieldAssign(...)     → compute memory addr + I32Store(offset)
        //   Stmt::Expr(expr)           → emit_expr(expr) (discard result)
        todo!()
    }

    fn emit_expr(&mut self, _expr: &Expr, _instrs: &mut Vec<Instruction>) {
        // TODO: match on expr variant:
        //   Expr::IntLit(n)            → I32Const(n)
        //   Expr::FloatLit(f)          → F64Const(f)
        //   Expr::StringLit(s)         → store in data section, push (offset, len)
        //   Expr::Var(name)            → LocalGet(slot)
        //   Expr::BinOp(l, op, r)      → emit l, emit r, emit op instruction
        //   Expr::Call(name, args)      → emit each arg, Call(func_index)
        //   Expr::FieldAccess(obj, f)   → emit obj addr, I32Load(field_offset)
        todo!()
    }

    fn op_to_instruction(&self, _op: &Op, _ty: &Type) -> Instruction {
        // TODO: map (Op, Type) → I32Add / I32Sub / F64Mul / etc.
        todo!()
    }

    fn add_string_literal(&mut self, _s: &str) -> (u32, u32) {
        // TODO: append bytes to data_sections, return (offset, length)
        todo!()
    }
}
