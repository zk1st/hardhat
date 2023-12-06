#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JumpType {
    NotJump,
    JumpIntoFunction,
    JumpOutOfFunction,
    InternalJump,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SourceMap {
    pub file: u32,
    pub offset: u32,
    pub length: u32,
    pub jump_type: JumpType,
}

impl SourceMap {
    pub fn contains(&self, other: &SourceMap) -> bool {
        self.file == other.file
            && self.offset <= other.offset
            && self.offset + self.length >= other.offset + other.length
    }

    // starting line?
}
