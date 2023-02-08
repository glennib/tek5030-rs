use anyhow::Result;
use clap::Parser;
use std::fs::read_to_string;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(help = "file of where to get the conan build information")]
    conan_build_info_file: String,
    #[arg(short = 'i', help = "variable to write the include directory list to")]
    include_dirs_var: Option<String>,
    #[arg(long = "ai", help = "append these include directories")]
    include_dirs_append: Vec<String>,
    #[arg(short = 'd', help = "variable to write the library directory list to")]
    lib_dirs_var: Option<String>,
    #[arg(long = "ad", help = "append these library directories")]
    lib_dirs_append: Vec<String>,
    #[arg(short = 'l', help = "variable to write the library list to")]
    libs_var: Option<String>,
    #[arg(long = "al", help = "append these libraries")]
    libs_append: Vec<String>,
    #[arg(short = 's', help = "include system libraries in libs_var")]
    include_system_libs: bool,
}

struct SectionContents(Vec<String>);

const INCLUDE_DIRS_HEADER: &str = "includedirs";
const LIB_DIRS_HEADER: &str = "libdirs";
const LIBS_HEADER: &str = "libs";
const SYSTEM_LIBS_HEADER: &str = "system_libs";

impl SectionContents {
    fn find(header: &str, haystack: &str) -> Option<Self> {
        haystack
            .split_once(&format!("[{header}]\n"))
            .map(|(_, after)| {
                let contents = after
                    .lines()
                    .take_while(|&line| !line.trim().is_empty() && !line.starts_with('['))
                    .map(std::string::ToString::to_string)
                    .collect();
                Self(contents)
            })
    }
}

fn print_variable<'a, S, A, Ap>(env: &str, sections: S, append: A) -> usize
where
    S: Iterator<Item = &'a SectionContents>,
    A: Iterator<Item = &'a Ap>,
    Ap: AsRef<str> + 'a,
{
    let contents: Vec<_> = sections
        .flat_map(|s| s.0.iter())
        .map(String::as_str)
        .chain(append.map(AsRef::as_ref))
        .collect();
    let number_of_items = contents.len();
    let contents = contents.join(",");
    println!("{env} = \"{contents}\"");
    number_of_items
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

    let n_include_dirs = if let Some(var) = cli.include_dirs_var {
        let section = SectionContents::find(INCLUDE_DIRS_HEADER, contents)
            .unwrap_or_else(|| panic!("contents should include {INCLUDE_DIRS_HEADER} section"));
        print_variable(
            &var,
            std::iter::once(&section),
            cli.include_dirs_append.iter(),
        )
    } else {
        0
    };

    let n_lib_dirs = if let Some(var) = cli.lib_dirs_var {
        let section = SectionContents::find(LIB_DIRS_HEADER, contents)
            .unwrap_or_else(|| panic!("contents should include {LIB_DIRS_HEADER} section"));
        print_variable(&var, std::iter::once(&section), cli.lib_dirs_append.iter())
    } else {
        0
    };

    let n_libs = if let Some(var) = cli.libs_var {
        let section = SectionContents::find(LIBS_HEADER, contents)
            .unwrap_or_else(|| panic!("contents should include {LIBS_HEADER} section"));
        let mut sections = vec![section];
        if cli.include_system_libs {
            let system = SectionContents::find(SYSTEM_LIBS_HEADER, contents)
                .unwrap_or_else(|| panic!("contents should include {SYSTEM_LIBS_HEADER} section"));
            sections.push(system);
        }
        print_variable(&var, sections.iter(), cli.libs_append.iter())
    } else {
        0
    };

    eprintln!(
        "Registered \
        {n_include_dirs} include directories, \
        {n_lib_dirs} library directories, and \
        {n_libs} libraries."
    );

    Ok(())
}
