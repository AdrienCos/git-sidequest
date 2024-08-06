#!/usr/bin/env bats

setup_file() {
    export TMPDIR="/tmp/git-sidequest"
    TRASHDIR=/tmp/git-sidequest-trash
    mkdir -p $TMPDIR
    mkdir -p $TRASHDIR
    if [ "$(ls -A $TMPDIR)" ];
    then
        mv "$TMPDIR/"* "$TRASHDIR"
    fi
    CWD=$(pwd)
    cargo build
    export EXE_PATH="$CWD/target/debug/git-sidequest"

    # Setup the master test git repository
    MASTER_WORKDIR=$(mktemp -d --tmpdir="$TMPDIR" -t "base.XXXXXX")
    cd "$MASTER_WORKDIR" || return 1
    git init --quiet
    head --bytes=1000 /dev/urandom | base64 > file.txt
    git add file.txt
    git commit -m "Initial commit" --quiet
    git switch -c conflict-branch --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some conflicting content" --quiet
    git checkout master
    git switch -c onto-branch --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some content on the onto branch" --quiet
    git switch -c original-branch --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some content" --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some more content" --quiet
    export MASTER_WORKDIR
}

setup() {
    load 'lib/bats-support/load'
    load 'lib/bats-assert/load'

    # Clone the master repo
    WORKDIR=$(mktemp -d --tmpdir="$TMPDIR" -t "test.XXXXXX")
    rm -r "$WORKDIR"
    cp -r "$MASTER_WORKDIR" "$WORKDIR"
    cd "$WORKDIR" || return 1

    # Run the test
    head --bytes=1000 /dev/urandom | base64 > non-sidequest.txt
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    head --bytes=1000 /dev/urandom | base64 > sidequest.txt
    git add sidequest.txt

    # Output debug information
    echo "workdir: $WORKDIR"
}

@test "can run the base case" {
    run $EXE_PATH sidequest-branch --message "Commit message for sidequest"
    assert_success
    assert_output --partial "Sidequest successful!"
}

@test "can use the --onto flag" {
    run $EXE_PATH --onto onto-branch sidequest-branch -m "Commit message for onto-branch sidequest"
    assert_success
    assert_output --partial "Sidequest successful!"
}

@test "aborts if no changes are staged" {
    git restore --staged .
    run $EXE_PATH sidequest-branch -m "This should not be commited"
    assert_failure
    assert_output --partial "No staged changes found"
}

@test "aborts if the target branch already exists" {
    run $EXE_PATH master -m "This should not be commited"
    assert_failure
    assert_output --partial "Target branch already exists"
}

@test "aborts if the onto branch does not exist" {
    run $EXE_PATH sidequest-branch --onto missing-branch -m "This should not be commited"
    assert_failure
    assert_output --partial "Onto branch does not exist"
}

@test "aborts if the repo is currently in a rebase operation" {
    # Manually force a merge conflict during the rebase
    git add .
    git commit -m "This commit creates a conflict" --quiet
    run git rebase conflict-branch
    run $EXE_PATH sidequest-branch -m "This should not be commited"
    assert_failure
    assert_output --partial "An operation is already in progress"
}

@test "aborts if HEAD is not a branch" {
    # Checkout a detached HEAD
    git restore . --quiet
    git checkout HEAD~1 --quiet
    run $EXE_PATH sidequest-branch -m "This should not be commited"
    assert_failure
    assert_output --partial "HEAD is not the top of a local branch"
}

@test "aborts if the target branch name is invalid" {
    run $EXE_PATH ..sidequest-branch -m "This should not be commited"
    assert_failure
    assert_output --partial "Invalid branch name"
}
