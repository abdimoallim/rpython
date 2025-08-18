use crate::object::*;
use crate::opcode::*;

#[derive(Clone, Default, PartialEq)]
pub struct CodeObject {
    pub consts: Vec<PyObject>,
    pub names: Vec<String>,
    pub instructions: Vec<Op>,
    pub nested: Vec<CodeObject>,
}

impl CodeObject {
    pub fn debug_print(&self) {
        println!("Constants: {:?}", self.consts);
        println!("Names: {:?}", self.names);
        println!("Instructions:");

        for (i, op) in self.instructions.iter().enumerate() {
            println!("  {}: {}", i, op);
        }
    }
}
