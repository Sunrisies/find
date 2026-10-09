mod cli;
mod matcher;
mod models;
mod searcher;

use std::env;
use std::process::ExitCode;

use cli::{command_kind, parse_args, print_help};
use matcher::Matcher;
use models::CommandKind;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let prog = args.first().map(String::as_str).unwrap_or("find");

    match command_kind(&args) {
        CommandKind::Help => {
            print_help(prog);
            ExitCode::SUCCESS
        }
        CommandKind::Version => {
            println!("{} {}", env!("CARGO_PKG_NAME"), VERSION);
            ExitCode::SUCCESS
        }
        CommandKind::Run => match parse_args(&args) {
            Ok(parsed) => {
                let matcher = Matcher::new(&parsed.keyword, parsed.match_mode);
                let stats = searcher::search(&parsed.roots, &matcher, parsed.mode);

                for warning in &stats.warnings {
                    eprintln!("警告: {}", warning);
                }
                if stats.warnings.len() == 10 {
                    eprintln!("（还有更多警告被省略）");
                }

                if stats.warnings.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(e) => {
                eprintln!("错误: {}", e);
                eprintln!();
                print_help(prog);
                ExitCode::FAILURE
            }
        },
    }
}
