#![warn(clippy::all, clippy::pedantic, clippy::style)]

use clap::Parser;
mod app;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "Name of the branch to create", value_parser = utils::validate_branch_name)]
    branch: String,
    // TODO: Add '--no-verify' option
    // TODO: Add '-a/--add' option
    // TODO: Add '-m/--message' option
    // TODO: Add '-n/--dry-run' option
    // TODO: Add '--onto' option to designate the base branch
}

#[allow(clippy::too_many_lines)]
fn main() {
    let args = Args::parse();

    println!("Sidequest started : {}", args.branch);

    // Check if we are in a git repo
    let repo = utils::open_repository().unwrap();

    // Instantiate the app
    let mut app = app::App::new(repo);

    // Get the signature that will be used for the new commit
    let signature = match app.default_signature() {
        Ok(sign) => sign,
        Err(e) => {
            eprint!("Sidequest failed: {e}");
            return;
        }
    };

    // Accomplish a sidequest
    match app.run(&args.branch, Some(&signature)) {
        Ok(()) => {
            println!("Sidequest successful!");
        }
        Err(e) => {
            eprintln!("Sidequest failed: {e}");
        }
    }
}
