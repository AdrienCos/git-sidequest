use std::any::Any;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Name of the branch to create", value_parser = validate_branch_name)]
    // TODO: Add a validator to ensure the branch name is valid
    branch: String,
    // TODO: Add '--no-verify' option
    // TODO: Add '-a/--add' option
    // TODO: Add '-m/--message' option
    // TODO: Add '-n/--dry-run' option
}

fn validate_branch_name(branch_name: &str) -> Result<String, String> {
    match git2::Branch::name_is_valid(branch_name) {
        Ok(true) => Ok(branch_name.to_string()),
        _ => Err("Invalid branch name".to_string()),
    }
}

fn main() {
    let args = Args::parse();

    println!("Sidequest started : {}", args.branch);

    // TODO: Check if we are in a git repo

    // TODO: Check if some changes are staged

    // TODO: Check if the branch already exists locally

    // TODO: Check if the branch already exists remotely

    // TODO: Check if there are unstaged changes

    // TODO: Stash the unstaged changes

    // TODO: Stash the staged changes

    // TODO: Checkout master

    // TODO: Create branch

    // TODO: Checkout the new branch

    // TODO: Apply the stashed staged changes

    // TODO: Stage the changes

    // TODO: Start a commit

    // TODO: Open the editor to write the commit message

    // TODO: Check if the commit was successful

    // TODO: Checkout the original branch

    // TODO: Apply the stashed unstaged changes

    println!("Sidequest completed!")
}
