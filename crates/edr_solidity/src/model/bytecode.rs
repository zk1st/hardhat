use std::collections::HashMap;

use super::{
    opcode::{decode_opcode, Opcode},
    source_map::SourceMap,
};

#[derive(Debug)]
pub enum BytecodeType {
    Runtime,
    Deployment,
}

#[derive(Debug)]
pub struct ImmutableReference {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug)]
pub struct Bytecode {
    pub contract_id: usize,
    pub bytecode_type: BytecodeType,
    pub nodermalized_code: Vec<u8>,
    pub library_offsets: Vec<usize>,
    pub immutable_references: Vec<ImmutableReference>,
    pc_to_source_maps: HashMap<usize, SourceMap>,
}

impl Bytecode {
    pub fn get_instruction(&self, program_counter: usize) -> Instruction {
        let source_map = self.pc_to_source_maps.get(&program_counter).cloned();
        Instruction {
            opcode: decode_opcode(&self.nodermalized_code, program_counter),
            source_map,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub source_map: Option<SourceMap>,
}
