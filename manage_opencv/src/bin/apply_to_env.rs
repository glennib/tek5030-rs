use clap::Parser;
use std::{
    fmt::Display,
    fs::{read_to_string, File, OpenOptions},
    io::Write,
    path::Path,
};
use toml::{map::Entry, Table, Value};

/// Read TOML table and apply the key/values to the `env`-section of the provided Cargo config file.
#[derive(Debug, Parser)]
#[command(about)]
struct Cli {
    /// File to read TOML table from
    env_contents_toml_file: String,
    /// .cargo/config.toml file to apply env variables to
    cargo_config_toml_file: String,
}

fn create_or_get_file<P>(path: &P) -> (File, Option<String>)
where
    P: AsRef<Path> + Display,
{
    let contents = read_to_string(path).ok();
    if let Some(contents) = &contents {
        eprintln!(
            "File at {path} already exists and contains {} characters",
            contents.len()
        );
    } else {
        eprintln!("File at {path} does not already exist");
    }
    let file = OpenOptions::new()
        .write(true)
        .read(false)
        .append(false)
        .create(true)
        .truncate(true)
        .open(path)
        .expect("should be able to create or open");
    (file, contents)
}

fn main() {
    let cli = Cli::parse();

    let env_contents = read_to_string(cli.env_contents_toml_file)
        .expect("env contents should be a file")
        .parse::<Table>()
        .expect("env contents file should be a valid TOML table");

    let (mut cargo_config_file, cargo_config_contents) =
        create_or_get_file(&cli.cargo_config_toml_file);

    let cargo_config_contents = cargo_config_contents.unwrap_or_default();

    let mut cargo_config = cargo_config_contents
        .parse::<Table>()
        .expect("cargo config file should be a valid TOML table");

    let cargo_config_env = match cargo_config.entry("env") {
        Entry::Vacant(entry) => {
            eprintln!("Creating new `env` section");
            entry
                .insert(Value::Table(Table::new()))
                .as_table_mut()
                .expect("newly inserted table should be parsable as table")
        }
        Entry::Occupied(entry) => {
            eprintln!("Modifying existing `env` section");
            entry
                .into_mut()
                .as_table_mut()
                .expect("entry env should be a table")
        }
    };

    for (key, val) in env_contents {
        let val = val.as_str().expect("value should be a string");
        match cargo_config_env.entry(&key) {
            Entry::Vacant(entry) => {
                eprintln!("Creating new variable {key}");
                entry.insert(Value::String(val.into()));
            }
            Entry::Occupied(mut entry) => {
                eprintln!("Replacing existing variable {key}");
                entry.insert(val.into());
            }
        }
    }

    let new_cargo_config_content = cargo_config.to_string();
    let new_length = new_cargo_config_content.len();
    cargo_config_file
        .write_all(new_cargo_config_content.as_bytes())
        .expect("should be able to write to file");
    eprintln!(
        "Wrote {new_length} characters to {}",
        cli.cargo_config_toml_file
    );
}
