use super::source_location::SourceLocation;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JumpType {
    NotJump,
    JumpIntoFunction,
    JumpOutOfFunction,
    InternalJump,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SourceMap {
    pub location: SourceLocation,
    pub jump_type: JumpType,
}
