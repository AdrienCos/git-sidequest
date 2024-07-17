#![warn(clippy::all, clippy::pedantic, clippy::style)]

use clap::Parser;
use git2::{Repository, Signature};

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
    // TODO: Add '--onto' option to designate the base branch
}

fn validate_branch_name(branch_name: &str) -> Result<String, String> {
    match git2::Branch::name_is_valid(branch_name) {
        Ok(true) => Ok(branch_name.to_string()),
        _ => Err("Invalid branch name".to_string()),
    }
}

fn commit_on_head(
    repo: &mut Repository,
    signature: &Signature,
    msg: &str,
) -> Result<git2::Oid, git2::Error> {
    let oid = repo.index()?.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let parent_commit = repo.head()?.peel_to_commit()?;
    repo.commit(
        Some("HEAD"),
        signature,
        signature,
        msg,
        &tree,
        &[&parent_commit],
    )
}

fn checkout_branch(repo: &mut Repository, branch: &str) -> Result<(), git2::Error> {
    let (object, _) = repo.revparse_ext(branch)?;
    repo.checkout_tree(&object, None)?;
    repo.set_head(&("refs/heads/".to_string() + branch))
}

fn create_branch(repo: &mut Repository, branch: &str, target: &str) -> Result<(), git2::Error> {
    let (_, reference) = repo.revparse_ext(target)?;

    // Create target branch
    repo.branch(
        branch,
        &repo
            .find_commit(reference.unwrap().target().unwrap())
            .unwrap(),
        false,
    )
    .map(|_| ())
}

fn get_current_branch_name(repo: &Repository) -> Option<String> {
    Some(repo.head().ok()?.shorthand()?.to_owned())
}

fn stash_push(
    repo: &mut Repository,
    signature: &Signature,
    message: &str,
    ignore_staged_files: bool,
) -> Result<(), git2::Error> {
    repo.stash_save(
        signature,
        message,
        if ignore_staged_files {
            Some(git2::StashFlags::KEEP_INDEX | git2::StashFlags::INCLUDE_UNTRACKED)
        } else {
            Some(git2::StashFlags::DEFAULT)
        },
    )
    .map(|_| ())
}

fn stash_pop(repo: &mut Repository) -> Result<(), git2::Error> {
    repo.stash_apply(0, None)?;
    repo.stash_drop(0)
}

fn branch_exists(repo: &Repository, branch: &str) -> bool {
    if repo.find_branch(branch, git2::BranchType::Local).is_ok() {
        return true;
    }
    if repo.find_branch(branch, git2::BranchType::Remote).is_ok() {
        return true;
    }
    false
}
fn has_unstaged_changes(repo: &Repository) -> Result<bool, git2::Error> {
    Ok(repo.statuses(None)?.iter().any(|status| {
        status.status().intersects(
            git2::Status::WT_DELETED
                | git2::Status::WT_MODIFIED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_NEW
                | git2::Status::WT_TYPECHANGE,
        )
    }))
}

fn has_staged_changes(repo: &Repository) -> Result<bool, git2::Error> {
    Ok(repo.statuses(None)?.iter().any(|status| {
        status.status().intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        )
    }))
}

fn is_mid_operation(repo: &Repository) -> bool {
    repo.state() != git2::RepositoryState::Clean
}

fn default_signature(repo: &Repository) -> Result<Signature<'static>, git2::Error> {
    repo.signature()
}

#[allow(clippy::too_many_lines)]
fn main() {
    let args = Args::parse();

    println!("Sidequest started : {}", args.branch);

    // Check if we are in a git repo
    let mut repo = match Repository::discover(".") {
        Ok(repo) => {
            println!("Opening repo: {:?}", repo.path());
            repo
        }
        Err(e) => {
            eprintln!("Failed to open repo: {e}");
            return;
        }
    };

    // Get the default signature
    let signature = default_signature(&repo).unwrap();

    // Get name of the current branch
    let original_branch_name = get_current_branch_name(&repo).unwrap();

    // Make sure that the repository is not in an intermediate state (rebasing, merging, etc.)
    if is_mid_operation(&repo) {
        eprintln!("An operation is already in progress");
        return;
    }

    // Check if some changes are staged
    if !has_staged_changes(&repo).unwrap() {
        eprintln!("No staged changes");
        return;
    }

    // Check if the branch already exists locally
    if branch_exists(&repo, &args.branch) {
        eprintln!("Branch already exists");
        return;
    }

    // Check if there are unstaged changes
    let unstaged_changes = has_unstaged_changes(&repo).unwrap();

    // Stash the unstaged changes
    if unstaged_changes {
        stash_push(
            &mut repo,
            &signature,
            "git-sidequest: unstaged changes",
            true,
        )
        .unwrap();
    }

    // Stash the staged changes
    stash_push(
        &mut repo,
        &signature,
        "git-sidequest: staged changes",
        false,
    )
    .unwrap();

    // Create target branch
    create_branch(&mut repo, &args.branch, "master").unwrap();

    // Checkout target branch
    checkout_branch(&mut repo, &args.branch).unwrap();

    // Apply the stashed staged changes
    stash_pop(&mut repo).unwrap();

    // Start a commit
    commit_on_head(
        &mut repo,
        &signature,
        "Git Sidequest: Commit staged changes",
    )
    .unwrap();

    // Checkout the original branch
    checkout_branch(&mut repo, &original_branch_name).unwrap();

    // Apply the stashed unstaged changes
    // FIXME: sadly, we cannot create a stash that does not contain the staged files,
    // so this stash pop will also apply the staged changes
    // New architecture should be :
    // - commit staged changes
    // - stash unstaged changes
    // - create and checkout new branch
    // - cherry-pick the sidequest commit
    // - checkout original branch
    // - reset the sidequest commit
    // - apply the stashed unstaged changes
    if unstaged_changes {
        stash_pop(&mut repo).unwrap();
    }

    println!("Sidequest completed!");
}
