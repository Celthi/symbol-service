use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Error, Read};

#[derive(Serialize, Deserialize)]
pub struct LibEntry {
    pub path: String,
    pub note: String,
}
#[derive(Serialize, Deserialize)]
struct LibPaths {
    pub entries: Vec<LibEntry>,
}

pub fn read_lib_paths(config_path: &str) -> Result<Vec<LibEntry>, Error> {
    let mut file = File::open(config_path)?;
    let mut buf: String = String::new();
    let _data = file.read_to_string(&mut buf)?;
    let lib_paths: LibPaths = serde_json::from_str(&buf)?;
    Ok(lib_paths.entries)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_open_file() {
        let res = read_lib_paths("./config.json");
        assert_eq!(res.is_ok(), true);
    }
}
