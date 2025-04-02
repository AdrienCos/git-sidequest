use anyhow::{anyhow, bail, Result};
use git2::{AnnotatedCommit, RebaseOptions, Repository, Signature};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    process::Command,
};

use crate::constants::DEFAULT_COMMIT_MESSAGE;

pub struct App {
    repo: Repository,
}

impl App {
    pub fn new(repo: Repository) -> App {
        App { repo }
    }

    pub fn run(
        &mut self,
        target_branch: &str,
        onto_branch: &str,
        signature: Option<&Signature>,
        message: Option<&str>,
    ) -> Result<()> {
        // Get the default signature if none was provided
        let signature = match signature {
            Some(sign) => sign,
            None => &self.default_signature()?,
        };

        // Make sure that the repository is not in an intermediate state (rebasing, merging, etc.)
        if self.is_mid_operation() {
            return Err(anyhow!("An operation is already in progress",));
        }

        // Check if some changes are staged
        if !self.has_staged_changes()? {
            return Err(anyhow!("No staged changes found"));
        }

        // Check if the branch already exists
        if self.branch_exists(target_branch) {
            return Err(anyhow!("Target branch already exists",));
        }

        // Ensure the the 'onto' branch exists
        if !self.branch_exists(onto_branch) {
            return Err(anyhow!("Onto branch does not exist",));
        }

        // Get name of the current branch
        let original_branch_name = self.get_current_branch_name()?;

        // If not provided, get a commit message from the user
        let message = match message {
            Some(msg) => msg,
            None => &self.get_commit_msg_from_editor()?,
        };

        // Check if there are unstaged changes
        let unstaged_changes = self.has_unstaged_changes()?;

        // Create the target branch at HEAD
        self.create_branch(target_branch, "HEAD")?;

        // Checkout target branch
        self.checkout_branch(target_branch)?;

        // Commit the staged changes
        self.commit_on_head(signature, message)?;

        // Stash the unstaged changes
        if unstaged_changes {
            self.stash_push(signature, "git-sidequest: unstaged changes", true)?;
        }

        // Rebase or rollback the target branch on the master branch
        if self
            .rebase_branch(target_branch, &original_branch_name, onto_branch, signature)
            .is_ok()
        {
            // Checkout the original branch
            self.checkout_branch(&original_branch_name)?;
            // Apply the stashed unstaged changes
            if unstaged_changes {
                self.stash_pop()?;
            }
            Ok(())
        } else {
            self.rollback_failed_rebase(
                original_branch_name.as_str(),
                target_branch,
                unstaged_changes,
            )?;
            Err(anyhow!(
                "Failed to rebase the sidequest branch onto the target branch, operation aborted"
            ))
        }
    }

    fn rollback_failed_rebase(
        &mut self,
        original_branch_name: &str,
        target_branch: &str,
        should_pop_stash: bool,
    ) -> Result<()> {
        self.reset_to_head_parent()?;
        if should_pop_stash {
            self.stash_pop()?;
        }
        self.checkout_branch(original_branch_name)?;
        self.repo
            .find_branch(target_branch, git2::BranchType::Local)?
            .delete()?;
        Ok(())
    }

    fn reset_to_head_parent(&self) -> Result<()> {
        let head_ref = self.repo.find_reference("HEAD")?;
        let head_commit = head_ref.peel_to_commit()?;
        let target_commit = head_commit.parent(0)?.into_object();
        self.repo
            .reset(&target_commit, git2::ResetType::Mixed, None)?;
        Ok(())
    }

    fn commit_on_head(&self, signature: &Signature, msg: &str) -> Result<git2::Oid> {
        let oid = self.repo.index()?.write_tree()?;
        let tree: git2::Tree<'_> = self.repo.find_tree(oid)?;
        let parent_commit = self.repo.head()?.peel_to_commit()?;
        Ok(self.repo.commit(
            Some("HEAD"),
            signature,
            signature,
            msg,
            &tree,
            &[&parent_commit],
        )?)
    }

    fn checkout_branch(&self, branch: &str) -> Result<()> {
        let (object, _) = self.repo.revparse_ext(branch)?;
        self.repo.checkout_tree(&object, None)?;
        Ok(self.repo.set_head(&("refs/heads/".to_string() + branch))?)
    }

    fn create_branch(&self, branch: &str, target: &str) -> Result<()> {
        let target_commit = self.repo.find_reference(target)?.peel_to_commit()?;
        self.repo.branch(branch, &target_commit, false)?;
        Ok(())
    }

    fn get_current_branch_name(&self) -> Result<String> {
        let head_ref = self.repo.head()?;
        if !head_ref.is_branch() {
            bail!("HEAD is not the top of a local branch",);
        }
        let Some(branch_name) = head_ref.shorthand() else {
            bail!("Unable to get the name of the current branch",);
        };
        Ok(branch_name.to_owned())
    }

    fn stash_push(
        &mut self,
        signature: &Signature,
        message: &str,
        ignore_staged_files: bool,
    ) -> Result<()> {
        Ok(self
            .repo
            .stash_save(
                signature,
                message,
                if ignore_staged_files {
                    Some(git2::StashFlags::KEEP_INDEX | git2::StashFlags::INCLUDE_UNTRACKED)
                } else {
                    Some(git2::StashFlags::DEFAULT)
                },
            )
            .map(|_| ())?)
    }

    fn stash_pop(&mut self) -> Result<()> {
        self.repo.stash_apply(0, None)?;
        Ok(self.repo.stash_drop(0)?)
    }

    fn rebase_branch(
        &self,
        current: &str,
        target: &str,
        onto: &str,
        signature: &Signature,
    ) -> Result<()> {
        let current_annotated_commit = self.get_annotated_commit_from_branch(current)?;
        let target_annotated_commit = self.get_annotated_commit_from_branch(target)?;
        let onto_annotated_commit = self.get_annotated_commit_from_branch(onto)?;
        let mut rebase = self.repo.rebase(
            Some(&current_annotated_commit),
            Some(&target_annotated_commit),
            Some(&onto_annotated_commit),
            Some(RebaseOptions::new().inmemory(true)),
        )?;
        while let Some(op) = rebase.next() {
            if let Some(git2::RebaseOperationType::Pick) = op?.kind() {
                rebase.commit(None, signature, None)?;
            }
        }
        rebase.finish(Some(signature)).map_err(|err| anyhow!(err))
    }

    fn get_annotated_commit_from_branch(&self, branch_name: &str) -> Result<AnnotatedCommit> {
        let onto_ref = self
            .repo
            .find_branch(branch_name, git2::BranchType::Local)?
            .into_reference();
        Ok(self.repo.reference_to_annotated_commit(&onto_ref)?)
    }

    fn get_commit_msg_from_editor(&self) -> Result<String> {
        // Get the path of the .git directory
        let repo_path = self
            .repo
            .workdir()
            .ok_or(anyhow!("Unable to find the path to the .git directory"))?;
        let buffer_path = repo_path.join(".git").join("COMMIT_EDITMSG");

        // Create an empty COMMIT_MSG file in the directory
        let mut file = match File::create(&buffer_path) {
            Err(why) => Err(anyhow!(
                "Failed to create commit message buffer {}: {}",
                buffer_path.display(),
                why,
            )),
            Ok(f) => Ok(f),
        }?;

        // Write the default commit message comments to the file
        file.write_all(DEFAULT_COMMIT_MESSAGE.as_bytes())
            .map_err(|why| {
                anyhow!("Failed to write initial contents of the commit message: {why}")
            })?;

        // Open it with the user's $EDITOR, or fallback to vi
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));
        match Command::new(editor.clone()).arg(&buffer_path).status() {
            Err(why) => {
                return match why.kind() {
                    std::io::ErrorKind::NotFound => Err(anyhow!(
                        "{editor} was not found in your PATH, cannot edit commit message."
                    )),
                    _ => Err(anyhow!("Failed while writing commit message: {why}")),
                }
            }
            Ok(status) => {
                if status.success() {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Failed while writing commit message with status code: {status}"
                    ))
                }
            }
        }?;

        // Read the file
        let mut message = String::new();
        let mut file = match File::open(buffer_path) {
            Ok(f) => Ok(f),
            Err(why) => Err(anyhow!("Unable to read back commit message : {why}")),
        }?;
        match file.read_to_string(&mut message) {
            Ok(_) => Ok(()),
            Err(why) => Err(anyhow!("Error while reading commit message : {why}")),
        }?;

        // Delete lines starting with a #
        let lines_without_comments: String = message
            .trim()
            .split_inclusive('\n')
            .filter(|line| !line.starts_with('#'))
            .collect();

        // Check if at least one non-blank line remains
        if lines_without_comments.trim().is_empty() {
            return Err(anyhow!("Empty commit message, aborting",));
        }

        // Return its contents, or error if the file is empty
        Ok(lines_without_comments)
    }

    fn branch_exists(&self, branch: &str) -> bool {
        if self
            .repo
            .find_branch(branch, git2::BranchType::Local)
            .is_ok()
        {
            return true;
        }
        if self
            .repo
            .find_branch(branch, git2::BranchType::Remote)
            .is_ok()
        {
            return true;
        }
        false
    }
    fn has_unstaged_changes(&self) -> Result<bool> {
        Ok(self.repo.statuses(None)?.iter().any(|status| {
            status.status().intersects(
                git2::Status::WT_DELETED
                    | git2::Status::WT_MODIFIED
                    | git2::Status::WT_RENAMED
                    | git2::Status::WT_NEW
                    | git2::Status::WT_TYPECHANGE,
            )
        }))
    }

    fn has_staged_changes(&self) -> Result<bool> {
        Ok(self.repo.statuses(None)?.iter().any(|status| {
            status.status().intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED
                    | git2::Status::INDEX_TYPECHANGE,
            )
        }))
    }

    fn is_mid_operation(&self) -> bool {
        self.repo.state() != git2::RepositoryState::Clean
    }

    pub fn default_signature(&self) -> Result<Signature<'static>> {
        Ok(self.repo.signature()?)
    }
}
