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

    let signature = if let Ok(signature) = repo.signature() {
        println!(
            "Using signature: {:?}<{:?}>",
            signature.name(),
            signature.email()
        );
        signature
    } else {
        eprintln!("Failed to get signature from repo");
        return;
    };

    // Get references to the current branch
    let original_branch_name = {
        let binding = git2::Branch::wrap(repo.head().unwrap().resolve().unwrap());
        binding.name().unwrap().unwrap().to_owned()
    };

    // Make sure that the repository is not in an intermediate state (rebasing, merging, etc.)
    if repo.state() == git2::RepositoryState::Clean {
        println!("Repository is clean");
    } else {
        eprintln!("Repository is not clean");
        return;
    }

    // Check if some changes are staged
    if repo.statuses(None).unwrap().iter().any(|status| {
        status.status().intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        )
    }) {
        println!("Some changes are staged");
    } else {
        eprintln!("No changes are staged");
        return;
    }

    // Check if the branch already exists locally
    if repo
        .find_branch(&args.branch, git2::BranchType::Local)
        .is_ok()
    {
        eprintln!("Branch already exists locally");
        return;
    }

    // Check if the branch already exists remotely
    if repo
        .find_branch(&args.branch, git2::BranchType::Remote)
        .is_ok()
    {
        eprintln!("Branch already exists remotely");
        return;
    }

    // Check if there are unstaged changes
    let unstaged_changes = if repo.statuses(None).unwrap().iter().any(|status| {
        status.status().intersects(
            git2::Status::WT_DELETED
                | git2::Status::WT_MODIFIED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_NEW
                | git2::Status::WT_TYPECHANGE,
        )
    }) {
        println!("Some changes are unstaged");
        true
    } else {
        println!("No changes are unstaged");
        false
    };

    // Stash the unstaged changes
    if unstaged_changes {
        match repo.stash_save(
            &signature,
            "git-sidequest: stash unstaged changes",
            Some(git2::StashFlags::KEEP_INDEX | git2::StashFlags::INCLUDE_UNTRACKED),
        ) {
            Ok(_) => {
                println!("Stashed unstaged changes");
            }
            Err(e) => {
                eprintln!("Failed to stash unstaged changes: {e}");
                return;
            }
        }
    }

    // Stash the staged changes
    match repo.stash_save(
        &signature,
        "git-sidequest: stash staged changes",
        Some(git2::StashFlags::DEFAULT),
    ) {
        Ok(_) => {
            println!("Stashed staged changes");
        }
        Err(e) => {
            eprintln!("Failed to stash staged changes: {e}");
            return;
        }
    }

    // Get references to the master branch
    {
        let (_, reference) = repo.revparse_ext("master").expect("Object not found");

        // Create target branch
        if let Err(error) = repo.branch(
            &args.branch,
            &repo
                .find_commit(reference.unwrap().target().unwrap())
                .unwrap(),
            false,
        ) {
            eprintln!("Failed to create target branch: {error}");
            return;
        }
        println!("Created branch: {}", args.branch);
    }
    // Checkout target branch
    checkout_branch(&mut repo, &args.branch).unwrap();

    // Apply the stashed staged changes
    if let Err(error) = repo.stash_apply(0, None) {
        eprintln!("Failed to apply stashed staged changes: {error}");
        return;
    }
    if let Err(error) = repo.stash_drop(0) {
        eprintln!("Failed to drop stashed staged changes: {error}");
        return;
    }

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
        match repo.stash_pop(0, None) {
            Ok(()) => {
                println!("Unstashed unstaged changes");
            }
            Err(e) => {
                eprintln!("Failed to unstash unstaged changes: {e}");
                return;
            }
        }
    }

    println!("Sidequest completed!");
}
