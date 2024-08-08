#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use anyhow::{bail, Context, Result};
use clap::Parser;
use usecases::InitialStateChecker;

mod app;
mod constants;
mod secondary_ports;
mod usecases;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "Name of the branch to create", value_parser = utils::validate_branch_name)]
    branch: String,
    #[arg(
        long,
        help = "Existing onto which to create the new branch",
        default_value = "master"
    )]
    onto: String,
    #[arg(short, long, help = "Commit message")]
    message: Option<String>,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let args = Args::parse();

    println!("Sidequest started: {}", args.branch);

    // Check if we are in a git repo
    let repo = utils::open_repository().context("No valid Git repository found")?;

    // Instantiate the usecases
    let initial_state_checker = Box::new(InitialStateChecker);

    // Instantiate the app
    let mut app = app::App::new(repo, initial_state_checker);

    // Get the signature that will be used for the new commit
    let signature = match app.default_signature() {
        Ok(sign) => sign,
        Err(e) => {
            bail!(e);
        }
    };

    let message = args.message.as_deref();

    // Accomplish a sidequest
    match app.run(&args.branch, &args.onto, Some(&signature), message) {
        Ok(()) => {
            println!("Sidequest successful!");
            Ok(())
        }
        Err(e) => {
            bail!(e);
        }
    }
}
