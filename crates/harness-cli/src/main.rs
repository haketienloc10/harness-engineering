mod application;
mod domain;
mod infrastructure;
mod interface;

use clap::Parser;
use std::ffi::OsStr;

fn main() {
    let json_requested = std::env::args_os().any(|argument| argument == OsStr::new("--json"));
    let cli = match interface::Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            use clap::error::ErrorKind;
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                let _ = error.print();
                std::process::exit(0);
            }
            if json_requested {
                let result = interface::usage_error_result(&error);
                interface::render_error(&result, true);
                std::process::exit(2);
            }
            let _ = error.print();
            std::process::exit(error.exit_code());
        }
    };
    let json = json_requested || cli.requests_json();
    if let Err(error) = interface::run(cli) {
        interface::render_error(&error.structured_result(), json);
        std::process::exit(error.exit_code());
    }
}
