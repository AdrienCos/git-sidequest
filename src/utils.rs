use anyhow::{anyhow, Result};
use git2::Repository;
use std::env::current_dir;

pub fn validate_branch_name(branch_name: &str) -> Result<String> {
    match git2::Branch::name_is_valid(branch_name) {
        Ok(true) => Ok(branch_name.to_string()),
        _ => Err(anyhow!("Invalid branch name")),
    }
}

pub fn open_repository() -> Result<Repository> {
    let cur_dir = current_dir()?;
    Ok(Repository::discover(cur_dir)?)
}
