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

pub fn uncompress_source_maps(raw_source_maps: &str) -> Vec<SourceMap> {
    let mut source_maps = Vec::new();

    // We need a default value for the first source map,
    // see: https://github.com/nomicfoundation/hardhat/issues/593
    let mut last_source_map = SourceMap {
        location: SourceLocation {
            file: None,
            offset: 0,
            length: 0,
        },
        jump_type: JumpType::NotJump,
    };

    for raw_source_map in raw_source_maps.split(';') {
        let raw_parts: Vec<&str> = raw_source_map.split(':').collect();

        let offset = raw_parts.get(0).and_then(|part| part.parse::<u32>().ok());
        let length = raw_parts.get(1).and_then(|part| part.parse::<u32>().ok());
        let file = raw_parts.get(2).and_then(|part| part.parse::<u32>().ok());
        let jump_type = raw_parts
            .get(3)
            .and_then(|part| part.chars().next())
            .map(|c| match c {
                'i' => JumpType::JumpIntoFunction,
                'o' => JumpType::JumpOutOfFunction,
                _ => JumpType::NotJump,
            });

        let source_map = SourceMap {
            location: SourceLocation {
                offset: offset.unwrap_or(last_source_map.location.offset),
                length: length.unwrap_or(last_source_map.location.length),
                file: file.or(last_source_map.location.file),
            },
            jump_type: jump_type.unwrap_or(last_source_map.jump_type),
        };

        last_source_map = source_map.clone();
        source_maps.push(source_map);
    }

    source_maps
}
