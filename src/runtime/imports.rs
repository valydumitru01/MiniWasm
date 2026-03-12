use std::collections::HashMap;
use crate::runtime::function::HostFunction;
use crate::runtime::memory::Memory;

pub enum Extern {
    Function(HostFunction),
    Memory(Memory),
}

pub struct Imports {
    map: HashMap<(String, String), Extern>,  // (namespace, name) → extern
}
impl Imports {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn define(&mut self, ns: &str, name: &str, val: impl Into<Extern>){
        self.map.insert((ns.to_string(), name.to_string()), val.into());
    }
    pub fn resolve(&self, module: &str, name: &str) -> Option<&Extern> {
        self.map.get(&(module.to_string(), name.to_string()))
    }
}