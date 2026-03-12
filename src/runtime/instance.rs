use crate::backend::executable_memory::ExecutableMemory;
use crate::runtime::imports::{Extern, Imports};
use crate::runtime::memory::Memory;
use crate::runtime::module::CompiledModule;
use crate::runtime::store::Store;
use std::collections::HashMap;

struct Instance {
    module: CompiledModule,
    import_pointers: Vec<*const u8>, // resolved import fn pointers
    executable: ExecutableMemory,    // mmap'd native code
    entry_points: HashMap<String, *const u8>, // export name → code ptr
    memory: *mut Memory,             // pointer to linear memory
}
impl Instance {
    fn new(store: &mut Store, module: CompiledModule, imports: &Imports) -> anyhow::Result<Self> {
        let mut import_pointers = Vec::new();
        for wasm_import in &module.wasm.imports {
            let ext = imports
                .resolve(&wasm_import.module, &wasm_import.name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "unresolved import: {}.{}",
                        wasm_import.module,
                        wasm_import.name
                    )
                })?;
            match ext {
                Extern::Function(host_fn) => {
                    import_pointers.push(host_fn.trampoline);
                }
                _ => {
                    anyhow::bail!(
                        "expected function import for {}.{}",
                        wasm_import.module,
                        wasm_import.name
                    );
                }
            }
        }

        let program_memory: Vec<u8> = module.functions.iter().flat_map(|f| f.code).collect();
        let executable = ExecutableMemory::new(&program_memory);
        let memory = Memory::new(module.memory_pages);
        Ok(Self {
            module,
            import_pointers,
            executable,
            entry_points: Default::default(),
            memory,
        })
    }
    pub fn call(&self, name: &str, args: &[i64]) -> anyhow::Result<i64> {
        let func = /* look up by name */;

        // Validate before jumping into JIT code
        if args.len() != func.func_type.params.len() {
            anyhow::bail!(
            "{}() expects {} args, got {}",
            name, func.func_type.params.len(), args.len()
        );
        }

        // Safe to call now
        unsafe { /* transmute and call */ }
    }
}
