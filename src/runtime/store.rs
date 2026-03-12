use crate::backend::compiler::Compiler;
use crate::runtime::memory::Memory;

pub struct Store {
    compiler: Box<dyn Compiler>,
    memory: Memory,
}