use std::{env, path::PathBuf};

use clap::Parser;
use hunspell::Config;

#[derive(Parser)]
struct Cli {
    /// Directory to place the default config into
    #[arg(default_value_t = path_default())]
    path: String,
    ///
    #[arg(long, action, default_value_t = overwrite_default())]
    overwrite: bool,
}

fn path_default() -> String {
    let homedir = env::var("HOME").expect("$HOME exists and is unicode");
    format!("{homedir}/.config/anyrun/")
}

fn overwrite_default() -> bool {
    false
}

fn main() {
    let cli = Cli::parse();
    let path = PathBuf::from(cli.path);
    if !path.is_dir() {
        eprintln!("Path {path:?} must exist and be a directory!");
        return;
    }

    let file_path = path.join("hunspell.ron");
    let file_exists = file_path.exists();

    match (file_exists, cli.overwrite) {
        (true, false) => {
            eprintln!(
                "File {file_path:?} already exists! Double check the path or use --overwrite to replace it."
            );
            return;
        }
        (true, true) if !file_path.is_file() => {
            eprintln!("Path {file_path:?} must be a file to be able to overwrite it!");
            return;
        }
        (true, true) | (false, _) => (),
    }
    let pretty_cfg = ron::ser::PrettyConfig::default()
        .struct_names(true)
        .separate_tuple_members(true);
    let config_str = ron::ser::to_string_pretty(&Config::default(), pretty_cfg)
        .expect("config can be serialized");
    std::fs::write(&file_path, config_str).expect("failed to write to file");

    println!("Successfully wrote default config to {file_path:?}");
}
