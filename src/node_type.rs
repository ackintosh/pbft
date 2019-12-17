#[derive(Debug, PartialEq)]
pub enum NodeType {
    Primary,
    Backup,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NodeType::Primary => "Primary",
            NodeType::Backup => "Backup",
        };
        write!(f, "{}", s)
    }
}

