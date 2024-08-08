use anyhow::Result;
use git2::Repository;

pub trait ICheckInitialState {
    fn check(&self, repo: &Repository, target_branch: &str, onto_branch: &str) -> Result<()>;
}
