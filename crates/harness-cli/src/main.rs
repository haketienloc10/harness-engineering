mod application;
mod domain;
mod infrastructure;
mod interface;

use clap::Parser;

fn main() {
    let cli = interface::Cli::parse();
    if let Err(error) = interface::run(cli) {
        eprintln!("error: {error}");
        let exit_code = match &error {
            interface::InterfaceError::Infrastructure(
                infrastructure::HarnessInfraError::UnsafeDurableState(_),
            ) => 3,
            interface::InterfaceError::Infrastructure(
                infrastructure::HarnessInfraError::BackupFailed(_)
                | infrastructure::HarnessInfraError::Sqlite(_),
            ) => 4,
            interface::InterfaceError::Infrastructure(
                infrastructure::HarnessInfraError::TaskIdentityPairRequired
                | infrastructure::HarnessInfraError::InvalidTaskLeaseDuration,
            ) => 2,
            interface::InterfaceError::Infrastructure(
                infrastructure::HarnessInfraError::TaskOwnerConflict { .. }
                | infrastructure::HarnessInfraError::TaskOwnerMismatch { .. }
                | infrastructure::HarnessInfraError::TaskOwnerRequired(_)
                | infrastructure::HarnessInfraError::TaskSessionRequired(_)
                | infrastructure::HarnessInfraError::TaskSessionMismatch { .. }
                | infrastructure::HarnessInfraError::TaskLeaseExpired
                | infrastructure::HarnessInfraError::TaskLeaseConflict { .. }
                | infrastructure::HarnessInfraError::TaskHandoffSameOwner
                | infrastructure::HarnessInfraError::TaskHandoffSameSession,
            ) => 8,
            _ => 1,
        };
        std::process::exit(exit_code);
    }
}
