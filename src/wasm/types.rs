#[derive(Clone)]
pub(crate) enum ValType {
    I32,
    F64,
}

#[derive(Clone)]
pub struct FuncType {
    params: Vec<ValType>,
    results: Vec<ValType>,
}

#[derive(Clone)]
pub enum Instruction {
    I32Const(i32),
    F64Const(f64),
    LocalGet(u32),
    LocalSet(u32),
    I32Add, I32Sub, I32Mul, I32Div,
    F64Add, F64Sub, F64Mul, F64Div,
    Call(u32),           // function index (imports first, then locals)
    Return,
    I32Load(u32),        // offset — for struct field access
    I32Store(u32),       // offset
    MemoryGrow,
}

pub struct WasmFunction {
    pub name: String,
    pub func_type: FuncType,
    pub locals: Vec<ValType>,
    pub body: Vec<Instruction>,
}

pub struct WasmImport {
    pub(crate) module: String,      // e.g. "env"
    pub(crate) name: String,        // e.g. "print_i32"
    pub(crate) func_type: FuncType,
}

pub struct WasmExport { name: String, func_index: u32 }

pub struct DataSection {
    offset: u32,         // memory offset
    data: Vec<u8>,      // string literal bytes
}

pub struct WasmModule {
    pub imports: Vec<WasmImport>,
    pub functions: Vec<WasmFunction>,
    pub exports: Vec<WasmExport>,
    pub data_sections: Vec<DataSection>,  // for string literals
    pub memory_pages: u32,                // initial memory size
}