use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::Result,
    path::Path,
    process::Command,
};

use clap::{arg, command};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use shlex::Shlex;

const UNREACHABLE_MESSAGE: &str = "commands were built over map's values, shouldn't reach here.";
const PRINT_COMMAND: &str = "print";
const PRINT_COMMAND_ABOUT: &str = "Print the command to run with the command passed as argument";
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

fn execute_command(command: &str) {
    let mut lex = Shlex::new(command);
    let mut args = lex.by_ref().collect::<VecDeque<_>>();
    if lex.had_error {
        panic!("Lex had errors");
    }

    let command_name = args
        .pop_front()
        .unwrap_or_else(|| panic!("Command '{command}' should not be empty"));

    if let Err(status) = Command::new(command_name).args(args).status() {
        panic!("Error status: {}", status);
    };
}

fn main() {
    let local_config = pull_config(File::open(Path::new(LOCAL_CONFIG_FILE)));
    let mut global_config = HashMap::from([]);
    GLOBAL_CONFIG_FILES
        .iter()
        .for_each(|config| global_config.extend(pull_config(File::open(Path::new(config)))));
    global_config.extend(local_config);

    let mut subcommands: Vec<clap::Command> = global_config
        .clone()
        .into_iter()
        .map(|(key, _)| clap::Command::new(key))
        .collect();
    subcommands.push(
        clap::Command::new(PRINT_COMMAND)
            .about(PRINT_COMMAND_ABOUT)
            .arg(arg!([COMMAND])),
    );

    let app = command!() // requires `cargo` feature
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands(subcommands);

    match app.get_matches().subcommand() {
        Some((PRINT_COMMAND, arg)) => println!(
            "{}",
            global_config
                .get(arg.get_one::<String>("COMMAND").unwrap())
                .unwrap_or_else(|| panic!("Command not found for printing."))
        ),
        Some((command, _)) => execute_command(
            global_config
                .get(command)
                .unwrap_or_else(|| unreachable!("{}", UNREACHABLE_MESSAGE)),
        ),
        None => unreachable!("{}", UNREACHABLE_MESSAGE),
    };
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
