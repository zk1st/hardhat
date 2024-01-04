use std::{collections::HashMap, sync::Arc};

use edr_eth::Bytes;
use revm::interpreter::OpCode;

use super::{source_map::SourceMap, uncompress_source_maps, Contract};
use crate::opcodes::opcode_length;

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
        raw_source_maps: Option<&str>,
    ) -> Self {
        let uncompressed_source_maps = uncompress_source_maps(raw_source_maps.unwrap_or_default());
        let pc_to_source_maps = build_pc_to_source_maps(&normalized_code, uncompressed_source_maps);

        Self {
            contract,
            bytecode_type,
            normalized_code,
            library_offsets,
            immutable_references,
            pc_to_source_maps,
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

fn build_pc_to_source_maps(
    normalized_code: &Bytes,
    uncompressed_source_maps: Vec<SourceMap>,
) -> HashMap<usize, SourceMap> {
    let mut pc_to_source_maps = HashMap::new();

    let mut bytes_index = 0;
    for (instruction_offset, source_map) in uncompressed_source_maps.into_iter().enumerate() {
        pc_to_source_maps.insert(bytes_index, source_map);

        let opcode = normalized_code
            .get(instruction_offset)
            .expect("the number of source maps is bigger than the length of the bytecode");

        bytes_index += opcode_length(*opcode);
    }

    pc_to_source_maps
}
