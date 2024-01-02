use std::{collections::HashMap, sync::Arc};

use edr_eth::Bytes;
use revm::interpreter::OpCode;

use super::{source_map::SourceMap, Contract};

#[derive(Debug, PartialEq)]
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
    pub contract: Option<Arc<Contract>>,
    pub bytecode_type: BytecodeType,
    pub normalized_code: Bytes,
    pub library_offsets: Vec<usize>,
    pub immutable_references: Vec<ImmutableReference>,
    pc_to_source_maps: HashMap<usize, SourceMap>,
}

impl Bytecode {
    pub fn new(
        contract: Option<Arc<Contract>>,
        bytecode_type: BytecodeType,
        normalized_code: Bytes,
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

    pub fn get_instruction(&self, program_counter: usize) -> Option<Instruction> {
        let source_map = self.pc_to_source_maps.get(&program_counter).cloned();

        self.normalized_code
            .get(program_counter)
            .and_then(|opcode| revm::interpreter::opcode::OpCode::new(*opcode))
            .map(|opcode| Instruction { opcode, source_map })
    }

    pub fn is_deployment(&self) -> bool {
        self.bytecode_type == BytecodeType::Deployment
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
    pub opcode: OpCode,
    pub source_map: Option<SourceMap>,
}
