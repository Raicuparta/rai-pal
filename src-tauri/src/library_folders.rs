use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LibraryFolder {
    pub path: String,
    pub apps: HashMap<String, String>
}

pub type LibraryFolders = HashMap<String, LibraryFolder>;
