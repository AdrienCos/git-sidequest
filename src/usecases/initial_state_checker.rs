use anyhow::{anyhow, Result};
use git2::Repository;

use crate::secondary_ports::ICheckInitialState;

pub struct InitialStateChecker;
impl ICheckInitialState for InitialStateChecker {
    fn check(&self, repo: &Repository, target_branch: &str, onto_branch: &str) -> Result<()> {
        if is_mid_operation(repo) {
            return Err(anyhow!("An operation is already in progress",));
        }

        // Check if some changes are staged
        if !has_staged_changes(repo)? {
            return Err(anyhow!("No staged changes found"));
        }

        // Check if the branch already exists
        if branch_exists(repo, target_branch) {
            return Err(anyhow!("Target branch already exists",));
        }

        // Ensure the the 'onto' branch exists
        if !branch_exists(repo, onto_branch) {
            return Err(anyhow!("Onto branch does not exist",));
        }
        Ok(())
    }
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

fn has_staged_changes(repo: &Repository) -> Result<bool> {
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
