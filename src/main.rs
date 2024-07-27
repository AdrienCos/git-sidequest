#![warn(clippy::all, clippy::pedantic, clippy::style)]

use clap::Parser;
use git_sidequest::lib;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Name of the branch to create", value_parser = lib::validate_branch_name)]
    // TODO: Add a validator to ensure the branch name is valid
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
    let mut repo = lib::open_repository().unwrap();

    // Get the default signature
    let signature = lib::default_signature(&repo).unwrap();

    // Get name of the current branch
    let original_branch_name = lib::get_current_branch_name(&repo).unwrap();

    // Make sure that the repository is not in an intermediate state (rebasing, merging, etc.)
    if lib::is_mid_operation(&repo) {
        eprintln!("An operation is already in progress");
        return;
    }

    // Check if some changes are staged
    if !lib::has_staged_changes(&repo).unwrap() {
        eprintln!("No staged changes");
        return;
    }

    // Check if the branch already exists locally
    if lib::branch_exists(&repo, &args.branch) {
        eprintln!("Branch already exists");
        return;
    }

    // Check if there are unstaged changes
    let unstaged_changes = lib::has_unstaged_changes(&repo).unwrap();

    // Create the target branch at HEAD
    lib::create_branch(&mut repo, &args.branch, "HEAD").unwrap();

    // Checkout target branch
    lib::checkout_branch(&mut repo, &args.branch).unwrap();

    // Commit the staged changes
    lib::commit_on_head(
        &mut repo,
        &signature,
        "Git Sidequest: Commit staged changes",
    )
    .unwrap();

    // Stash the unstaged changes
    if unstaged_changes {
        lib::stash_push(
            &mut repo,
            &signature,
            "git-sidequest: unstaged changes",
            true,
        )
        .unwrap();
    }
    // Rebase the target branch on the master branch
    lib::rebase_branch(
        &mut repo,
        &args.branch,
        &original_branch_name,
        "master",
        &signature,
    )
    .unwrap();

    // Checkout the original branch
    lib::checkout_branch(&mut repo, &original_branch_name).unwrap();

    // Apply the stashed unstaged changes
    if unstaged_changes {
        lib::stash_pop(&mut repo).unwrap();
    }

    println!("Sidequest completed!");
}
