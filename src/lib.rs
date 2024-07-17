pub mod lib {
    use git2::{Repository, Signature};

    pub fn validate_branch_name(branch_name: &str) -> Result<String, String> {
        match git2::Branch::name_is_valid(branch_name) {
            Ok(true) => Ok(branch_name.to_string()),
            _ => Err("Invalid branch name".to_string()),
        }
    }

    pub fn commit_on_head(
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

    pub fn checkout_branch(repo: &mut Repository, branch: &str) -> Result<(), git2::Error> {
        let (object, _) = repo.revparse_ext(branch)?;
        repo.checkout_tree(&object, None)?;
        repo.set_head(&("refs/heads/".to_string() + branch))
    }

    pub fn create_branch(
        repo: &mut Repository,
        branch: &str,
        target: &str,
    ) -> Result<(), git2::Error> {
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

    pub fn get_current_branch_name(repo: &Repository) -> Option<String> {
        Some(repo.head().ok()?.shorthand()?.to_owned())
    }

    pub fn stash_push(
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

    pub fn stash_pop(repo: &mut Repository) -> Result<(), git2::Error> {
        repo.stash_apply(0, None)?;
        repo.stash_drop(0)
    }

    pub fn branch_exists(repo: &Repository, branch: &str) -> bool {
        if repo.find_branch(branch, git2::BranchType::Local).is_ok() {
            return true;
        }
        if repo.find_branch(branch, git2::BranchType::Remote).is_ok() {
            return true;
        }
        false
    }
    pub fn has_unstaged_changes(repo: &Repository) -> Result<bool, git2::Error> {
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

    pub fn has_staged_changes(repo: &Repository) -> Result<bool, git2::Error> {
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

    pub fn is_mid_operation(repo: &Repository) -> bool {
        repo.state() != git2::RepositoryState::Clean
    }

    pub fn default_signature(repo: &Repository) -> Result<Signature<'static>, git2::Error> {
        repo.signature()
    }

    pub fn open_repository() -> Result<Repository, git2::Error> {
        Repository::discover(".")
    }
}
