use std::{collections::HashMap, sync::Arc};

use super::{
    opcode::{decode_opcode, Opcode},
    source_map::SourceMap,
    Contract,
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
    pub contract: Arc<Contract>,
    pub bytecode_type: BytecodeType,
    pub normalized_code: Vec<u8>,
    pub library_offsets: Vec<usize>,
    pub immutable_references: Vec<ImmutableReference>,
    pc_to_source_maps: HashMap<usize, SourceMap>,
}

impl Bytecode {
    pub fn new(
        contract: Arc<Contract>,
        bytecode_type: BytecodeType,
        normalized_code: Vec<u8>,
        library_offsets: Vec<usize>,
        immutable_references: Vec<ImmutableReference>,
    ) -> Self {
        Self {
            contract,
            bytecode_type,
            normalized_code,
            library_offsets,
            immutable_references,
            // TODO
            pc_to_source_maps: HashMap::new(),
        }
    }
    pub fn get_instruction(&self, program_counter: usize) -> Instruction {
        let source_map = self.pc_to_source_maps.get(&program_counter).cloned();
        Instruction {
            opcode: decode_opcode(&self.normalized_code, program_counter),
            source_map,
        }
    }
}

impl PartialEq for Bytecode {
    fn eq(&self, other: &Self) -> bool {
        self.normalized_code == other.normalized_code
    }
}

impl Eq for Bytecode {}

impl std::hash::Hash for Bytecode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normalized_code.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub source_map: Option<SourceMap>,
}
