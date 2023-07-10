use clap::{Parser, Subcommand};

mod client;
pub mod daemon;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Daemon mode
    Daemon,

    /// Get idle inhibitor status
    Status,

    /// Enable idle inhibitor
    Enable,

    /// Disable idle inhibitor
    Disable,

    /// Toggle idle inhibitor
    Toggle,
}

/// Needed for command and u8 conversion
impl From<u8> for Command {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Daemon,
            1 => Self::Status,
            2 => Self::Enable,
            3 => Self::Disable,
            4 => Self::Toggle,
            _ => panic!("Incorrect command value"),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Daemon => daemon::start_daemon(),
        cmd => client::start_client(cmd),
    }
}
