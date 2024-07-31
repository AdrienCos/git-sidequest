use std::{
    env, fmt,
    fs::File,
    io::{Read, Write},
    process::Command,
};

use git2::{Repository, Signature};

use crate::constants::DEFAULT_COMMIT_MESSAGE;

pub struct App {
    repo: Repository,
}

pub enum SidequestError {
    Git(git2::Error),
    App(String),
}

impl From<git2::Error> for SidequestError {
    fn from(e: git2::Error) -> Self {
        SidequestError::Git(e)
    }
}
impl fmt::Display for SidequestError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self {
            SidequestError::App(err) => write!(f, "{err}"),
            SidequestError::Git(err) => write!(f, "{err}"),
        }
    }
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
    ) -> Result<(), SidequestError> {
        // Get the default signature if none was provided
        let signature = match signature {
            Some(sign) => sign,
            None => &self.default_signature()?,
        };

        // Get name of the current branch
        let original_branch_name = self.get_current_branch_name().unwrap();

        // Make sure that the repository is not in an intermediate state (rebasing, merging, etc.)
        if self.is_mid_operation() {
            return Err(SidequestError::App(String::from(
                "An operation is already in progress",
            )));
        }

        // Check if some changes are staged
        if !self.has_staged_changes()? {
            return Err(SidequestError::App(String::from("No staged changes found")));
        }

        // Check if the branch already exists
        if self.branch_exists(target_branch) {
            return Err(SidequestError::App(String::from(
                "Target branch already exists",
            )));
        }

        // Ensure the the 'onto' branch exists
        if !self.branch_exists(onto_branch) {
            return Err(SidequestError::App(String::from(
                "Onto branch does not exist",
            )));
        }

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
        // Rebase the target branch on the master branch
        self.rebase_branch(target_branch, &original_branch_name, onto_branch, signature)?;

        // Checkout the original branch
        self.checkout_branch(&original_branch_name)?;

        // Apply the stashed unstaged changes
        if unstaged_changes {
            self.stash_pop()?;
        }
        Ok(())
    }

    fn commit_on_head(&self, signature: &Signature, msg: &str) -> Result<git2::Oid, git2::Error> {
        let oid = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(oid)?;
        let parent_commit = self.repo.head()?.peel_to_commit()?;
        self.repo.commit(
            Some("HEAD"),
            signature,
            signature,
            msg,
            &tree,
            &[&parent_commit],
        )
    }

    fn checkout_branch(&self, branch: &str) -> Result<(), git2::Error> {
        let (object, _) = self.repo.revparse_ext(branch)?;
        self.repo.checkout_tree(&object, None)?;
        self.repo.set_head(&("refs/heads/".to_string() + branch))
    }

    fn create_branch(&self, branch: &str, target: &str) -> Result<(), git2::Error> {
        let (_, reference) = self.repo.revparse_ext(target)?;

        // Create target branch
        self.repo
            .branch(
                branch,
                &self
                    .repo
                    .find_commit(reference.unwrap().target().unwrap())
                    .unwrap(),
                false,
            )
            .map(|_| ())
    }

    fn get_current_branch_name(&self) -> Option<String> {
        Some(self.repo.head().ok()?.shorthand()?.to_owned())
    }

    fn stash_push(
        &mut self,
        signature: &Signature,
        message: &str,
        ignore_staged_files: bool,
    ) -> Result<(), git2::Error> {
        self.repo
            .stash_save(
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

    fn stash_pop(&mut self) -> Result<(), git2::Error> {
        self.repo.stash_apply(0, None)?;
        self.repo.stash_drop(0)
    }

    fn rebase_branch(
        &self,
        current: &str,
        target: &str,
        onto: &str,
        signature: &Signature,
    ) -> Result<(), git2::Error> {
        let current_ref = self
            .repo
            .find_branch(current, git2::BranchType::Local)?
            .into_reference();
        let target_ref = self
            .repo
            .find_branch(target, git2::BranchType::Local)?
            .into_reference();
        let onto_ref = self
            .repo
            .find_branch(onto, git2::BranchType::Local)?
            .into_reference();
        let current_annotated_commit = self.repo.reference_to_annotated_commit(&current_ref)?;
        let target_annotated_commit = self.repo.reference_to_annotated_commit(&target_ref)?;
        let onto_annotated_commit = self.repo.reference_to_annotated_commit(&onto_ref)?;
        let mut rebase = self.repo.rebase(
            Some(&current_annotated_commit),
            Some(&target_annotated_commit),
            Some(&onto_annotated_commit),
            None,
        )?;
        while let Some(op) = rebase.next() {
            match op?.kind() {
                Some(git2::RebaseOperationType::Pick) => {
                    rebase.commit(None, signature, None).map(|_| ())?;
                }
                Some(_) | None => {}
            }
        }
        rebase.finish(Some(signature))
    }

    fn get_commit_msg_from_editor(&self) -> Result<String, SidequestError> {
        // Get the path of the .git directory
        let repo_path = self.repo.workdir().ok_or(SidequestError::App(String::from(
            "Unable to find the path to the .git directory",
        )))?;
        let buffer_path = repo_path.join(".git").join("COMMIT_EDITMSG");

        // Create an empty COMMIT_MSG file in the directory
        let mut file = match File::create(&buffer_path) {
            Err(why) => Err(SidequestError::App(format!(
                "Failed to create commit message buffer {}: {}",
                buffer_path.display(),
                why,
            ))),
            Ok(f) => Ok(f),
        }?;

        // Write the default commit message comments to the file
        file.write_all(DEFAULT_COMMIT_MESSAGE.as_bytes())
            .map_err(|why| {
                SidequestError::App(format!(
                    "Failed to write initial contents of the commit message: {why}"
                ))
            })?;

        // Open it with the user's $EDITOR, or fallback to vi
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));
        match Command::new(editor.clone()).arg(&buffer_path).status() {
            Err(why) => {
                return match why.kind() {
                    std::io::ErrorKind::NotFound => Err(SidequestError::App(format!(
                        "{editor} was not found in your PATH, cannot edit commit message."
                    ))),
                    _ => Err(SidequestError::App(format!(
                        "Failed while writing commit message: {why}"
                    ))),
                }
            }
            Ok(status) => {
                if status.success() {
                    Ok(())
                } else {
                    Err(SidequestError::App(format!(
                        "Failed while writing commit message with status code: {status}"
                    )))
                }
            }
        }?;

        // Read the file
        let mut message = String::new();
        let mut file = match File::open(buffer_path) {
            Ok(f) => Ok(f),
            Err(why) => Err(SidequestError::App(format!(
                "Unable to read back commit message : {why}"
            ))),
        }?;
        match file.read_to_string(&mut message) {
            Ok(_) => Ok(()),
            Err(why) => Err(SidequestError::App(format!(
                "Error while reading commit message : {why}"
            ))),
        }?;

        // Delete lines starting with a #
        let lines_without_comments: String = message
            .trim()
            .split_inclusive('\n')
            .filter(|line| !line.starts_with('#'))
            .collect();

        // Check if at least one non-blank line remains
        if lines_without_comments.trim().is_empty() {
            return Err(SidequestError::App(String::from(
                "Empty commit message, aborting",
            )));
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
    fn has_unstaged_changes(&self) -> Result<bool, git2::Error> {
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

    fn has_staged_changes(&self) -> Result<bool, git2::Error> {
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

    pub fn default_signature(&self) -> Result<Signature<'static>, git2::Error> {
        self.repo.signature()
    }
}
