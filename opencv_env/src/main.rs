use clap::{Parser, Subcommand};

#[allow(dead_code)]
mod build {
    include!("../../build/conan_cargo_build.rs");
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(short, long)]
    append: bool,
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    Print { line_prefix: Option<String> },
    Set,
}

impl Command {
    fn eprint(&self) {
        match self {
            Command::Print { line_prefix } => match line_prefix {
                None => {
                    eprintln!("Printing without line_prefix");
                }
                Some(line_prefix) => {
                    eprintln!("Printing with line prefix `{line_prefix}`");
                }
            },
            Command::Set => {
                eprintln!("Setting environment variables");
            }
        }
    }
}

struct Env {
    key: String,
    value: String,
}

impl Env {
    fn new(key: String, mut value: String, append: bool) -> Self {
        if append {
            value.insert(0, '+');
        }
        Self { key, value }
    }

    fn set(self) {
        std::env::set_var(self.key, self.value);
    }

    fn print(self, line_prefix: Option<&str>) {
        if let Some(line_prefix) = line_prefix {
            print!("{line_prefix} ");
        }
        println!("{}={}", self.key, self.value);
    }
}

fn main() {
    let cli = Cli::parse();

    let command = cli.command.unwrap_or(Command::Print { line_prefix: None });

    if !cli.quiet {
        command.eprint();
    }

    let envs = vec![
        Env::new("OPENCV_LINK_LIBS".into(), build::LIBS.join(","), cli.append),
        Env::new(
            "OPENCV_LINK_PATHS".into(),
            build::LIB_PATHS.join(","),
            cli.append,
        ),
        Env::new(
            "OPENCV_INCLUDE_PATHS".into(),
            build::INCLUDE_PATHS.join(","),
            cli.append,
        ),
    ];

    for env in envs {
        match &command {
            Command::Print { line_prefix } => env.print(line_prefix.as_deref()),
            Command::Set => env.set(),
        }
    }
}
