use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Result,
    path::Path,
};

use clap::Parser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const GLOBAL_CONFIG_PATH: &str = "dyncomp/config.json";
const LOCAL_CONFIG_FILE: &str = "dyncomp.json";
static GLOBAL_CONFIG_FILES: Lazy<HashSet<String>> = Lazy::new(|| {
    HashSet::from([
        format!(
            "{}/{}",
            dirs::config_dir()
                .unwrap_or_else(|| panic!("Can't find config_dir"))
                .display(),
            GLOBAL_CONFIG_PATH
        ),
        format!(
            "{}/.config/{}",
            dirs::home_dir()
                .unwrap_or_else(|| panic!("Can't find home_dir"))
                .display(),
            GLOBAL_CONFIG_PATH
        ),
    ])
});

#[derive(Debug, Serialize, Deserialize)]
struct DynCommands {
    commands: HashMap<String, String>,
}

/// CLI to run commands dynamically per project, easily configurable
/// with dyncomp.json configuration files.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Transactions file path.
    // #[clap(short, long)]
    // file: String,
}

fn pull_config(config_file: Result<File>) -> HashMap<String, String> {
    match config_file {
        Ok(file) => {
            serde_json::from_reader::<File, DynCommands>(file)
                .expect("deserialization of local config file to work.")
                .commands
        }
        Err(_) => HashMap::from([]),
    }
}

fn main() {
    let local_config = pull_config(File::open(Path::new(LOCAL_CONFIG_FILE)));
    dbg!(local_config);
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom, Write};

    use super::*;

    #[test]
    fn local_config_file() {
        let mut file = tempfile::tempfile().unwrap();
        write!(file, r#"{{"commands": {{"run": "test"}}}}"#).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let local_config = pull_config(Ok(file));
        assert_eq!(
            local_config,
            HashMap::from([("run".to_owned(), "test".to_owned())])
        );
    }

    #[test]
    fn no_local_config_file() {
        assert_eq!(
            pull_config(File::open(Path::new("unexistent file"))),
            HashMap::from([])
        );
    }
}
