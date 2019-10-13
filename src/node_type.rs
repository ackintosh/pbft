use crate::config::{Config, Port};

#[derive(Debug, PartialEq)]
enum NodeType {
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

#[derive(Debug)]
pub struct CurrentType {
    node_type: NodeType,
}

impl CurrentType {
    pub fn from(config: &Config, port: &Port) -> Result<CurrentType, ()> {
        if config.is_primary(port) {
            Ok(Self { node_type: NodeType::Primary })
        } else if config.is_backup(port) {
            Ok(Self { node_type: NodeType::Backup })
        } else {
            Err(())
        }
    }

    pub fn is_backup(&self) -> bool {
        self.node_type == NodeType::Backup
    }
}

impl std::fmt::Display for CurrentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.node_type)
    }
}