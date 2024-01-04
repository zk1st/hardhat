#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct SourceLocation {
    pub file: Option<u32>,
    pub offset: u32,
    pub length: u32,
}

impl SourceLocation {
    pub fn contains(&self, other: &SourceLocation) -> bool {
        self.file == other.file
            && self.offset <= other.offset
            && self.offset + self.length >= other.offset + other.length
    }
}
