use git2::Repository;

pub fn validate_branch_name(branch_name: &str) -> Result<String, String> {
    match git2::Branch::name_is_valid(branch_name) {
        Ok(true) => Ok(branch_name.to_string()),
        _ => Err("Invalid branch name".to_string()),
    }
}

pub fn open_repository() -> Result<Repository, git2::Error> {
    Repository::discover(".")
}
