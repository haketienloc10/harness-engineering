mod application;
mod domain;
mod infrastructure;
mod interface;

use clap::Parser;

fn main() {
    let cli = interface::Cli::parse();
    let json = cli.requests_json();
    if let Err(error) = interface::run(cli) {
        interface::render_error(&error.structured_result(), json);
        std::process::exit(error.exit_code());
    }
}
