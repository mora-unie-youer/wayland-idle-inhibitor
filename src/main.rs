use clap::{Parser, Subcommand};

mod daemon;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Daemon mode
    Daemon,

    // Get idle inhibitor status
    Status,

    /// Enable idle inhibitor
    Enable,

    /// Disable idle inhibitor
    Disable,

    /// Toggle idle inhibitor
    Toggle,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Daemon => daemon::start_daemon(),
        Command::Status => println!("Status"),
        Command::Enable => println!("Enable"),
        Command::Disable => println!("Disable"),
        Command::Toggle => println!("Toggle"),
    }
}
