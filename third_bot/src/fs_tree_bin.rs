use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize, Deserialize)]
pub struct FsNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub children: Option<Vec<FsNode>>, // Some para diretórios, None para arquivos
}

impl FsNode {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = fs::metadata(path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("/"));
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let children = if is_dir {
            let mut nodes = Vec::new();
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let child_path = entry.path();
                if let Ok(child_node) = FsNode::from_path(&child_path) {
                    nodes.push(child_node);
                }
            }
            Some(nodes)
        } else {
            None
        };
        Ok(FsNode {
            path: path.to_string_lossy().to_string(),
            name,
            is_dir,
            size,
            modified,
            children,
        })
    }
}

fn main() {
    let home = dirs::home_dir().unwrap();
    let tree = FsNode::from_path(&home).unwrap();
    let json = serde_json::to_string_pretty(&tree).unwrap();
    println!("{}", json);
}
