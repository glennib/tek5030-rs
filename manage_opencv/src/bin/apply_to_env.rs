use clap::Parser;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use toml::map::Entry;
use toml::{Table, Value};

#[derive(Debug, Parser)]
struct Cli {
    env_contents_toml_file: String,
    cargo_config_toml_file: String,
}

fn create_or_get_file<P>(path: &P) -> File
where
    P: AsRef<Path>,
{
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(true)
        .truncate(true)
        .open(path)
        .expect("should be able to create or open")
}

fn main() {
    let cli = Cli::parse();

    let env_contents = read_to_string(cli.env_contents_toml_file)
        .expect("env contents should be a file")
        .parse::<Table>()
        .expect("env contents file should be a valid TOML table");

    let mut cargo_config_file = create_or_get_file(&cli.cargo_config_toml_file);
    let mut cargo_config_contents = String::new();
    cargo_config_file
        .read_to_string(&mut cargo_config_contents)
        .expect("should be able to read from cargo config file into string");

    let mut cargo_config = cargo_config_contents
        .parse::<Table>()
        .expect("cargo config file should be a valid TOML table");

    let cargo_config_env = match cargo_config.entry("env") {
        Entry::Vacant(entry) => entry
            .insert(Value::Table(Table::new()))
            .as_table_mut()
            .expect("newly inserted table should be parsable as table"),
        Entry::Occupied(entry) => entry
            .into_mut()
            .as_table_mut()
            .expect("entry env should be a table"),
    };

    for (key, val) in env_contents {
        let val = val.as_str().expect("value should be a string");
        match cargo_config_env.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(Value::String(val.into()));
            }
            Entry::Occupied(mut entry) => {
                entry.insert(val.into());
            }
        }
    }

    cargo_config_file
        .write_all(cargo_config.to_string().as_bytes())
        .expect("should be able to write to file");

}
