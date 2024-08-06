use anyhow::{anyhow, Result};
use git2::Repository;

pub fn validate_branch_name(branch_name: &str) -> Result<String> {
    match git2::Branch::name_is_valid(branch_name) {
        Ok(true) => Ok(branch_name.to_string()),
        _ => Err(anyhow!("Invalid branch name")),
    }
}

pub fn open_repository() -> Result<Repository> {
    Ok(Repository::discover(".")?)
}
