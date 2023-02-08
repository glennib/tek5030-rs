use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Debug, Parser)]
struct Cli {
    conan_build_info_file: String,
    #[arg(short = 'i', help = "variable to write the include directory list to")]
    include_dirs_var: Option<String>,
    #[arg(short = 'd', help = "variable to write the library directory list to")]
    lib_dirs_var: Option<String>,
    #[arg(short = 'l', help = "variable to write the library list to")]
    libs_var: Option<String>,
    #[arg(short = 's', help = "include system libraries in libs_var")]
    include_system_libs: bool,
}

struct Section {
    header: String,
    contents: Vec<String>,
}

const INCLUDE_DIRS_HEADER: &str = "includedirs";
const LIB_DIRS_HEADER: &str = "libdirs";
const LIBS_HEADER: &str = "libs";
const SYSTEM_LIBS_HEADER: &str = "system_libs";

fn get_header(section: &str) -> &str {
    section
        .lines()
        .next()
        .expect("input should have at least one line")
        .strip_prefix('[')
        .expect("header should start with [")
        .strip_suffix(']')
        .expect("header should end with ]")
}

impl Section {
    fn from(s: &str) -> Self {
        let header = get_header(s).into();
        let contents = s
            .lines()
            .skip(1)
            .filter_map(|s| if s.is_empty() { None } else { Some(s.into()) })
            .collect();
        Self { header, contents }
    }
}

fn header_is_interesting(header: &str, header_env_map: &HashMap<&str, String>) -> bool {
    header_env_map.contains_key(header)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let contents = read_to_string(cli.conan_build_info_file)?;
    let contents = {
        contents
            .split_once("\n[USER_")
            .expect("should have a section starting with `\\n[USER_` to delimit interesting info")
            .0
    };

    let map = {
        let mut map = HashMap::new();
        if let Some(e) = cli.include_dirs_var {
            map.insert(INCLUDE_DIRS_HEADER, e);
        }
        if let Some(e) = cli.lib_dirs_var {
            map.insert(LIB_DIRS_HEADER, e);
        }
        if let Some(e) = cli.libs_var {
            map.insert(LIBS_HEADER, e);
        }
        map
    };

    if map.is_empty() {
        return Ok(());
    }

    let sections = contents.split("\n\n").filter_map(|section| {
        if section.is_empty() || !header_is_interesting(get_header(section), &map) {
            None
        } else {
            Some(Section::from(section))
        }
    });

    let mut remaining = map.len();
    for section in sections {
        let env_name = map
            .get(section.header.as_str())
            .expect("map should contain this header");
        let contents = section.contents.join(",");
        println!("{env_name} = \"{contents}\"");
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }

    Ok(())
}
