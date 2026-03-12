mod c_compiler;
mod runtime;
mod backend;
mod wasm;

use std::env::args;
use crate::c_compiler::{emit_wasm, parser};
use crate::runtime::imports::Imports;
use crate::runtime::module::CompiledModule;
use crate::runtime::store::Store;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();
    let source = std::fs::read_to_string(&args[1]).unwrap();

    // 1. C → Wasm IR
    let tokens = c_compiler::lexer::tokenize(&source);
    let program = parser::parse(tokens);
    let wasm_module = emit_wasm::emit(program);

    // 2. Set up imports (like WASI registration)
    let mut imports = Imports::new();
    imports.define("env", "print_i32", host_print_i32 as extern "C" fn(i32));
    imports.define("env", "print_str", host_print_str as extern "C" fn(i32, i32));

    // 3. Create store with x86_64 compiler
    let mut store = Store::new(Box::new(X86_64Compiler));

    // 4. Compile
    let module = CompiledModule::new(&store, &wasm_module, &imports)?;

    // 5. Instantiate (link imports)
    let instance = Instance::new(&mut store, module, &imports)?;

    // 6. Call main
    let result = instance.call("main", &[])?;
    std::process::exit(result as i32);
}