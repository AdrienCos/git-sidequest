#!/usr/bin/env sh

# Enable strict mode
set -eu

# Create a temporary directory with a random name in /tmp
TMPDIR="/tmp/git-sidequest"
TRASHDIR=/tmp/git-sidequest-trash
mkdir -p $TMPDIR
mkdir -p $TRASHDIR
if [ "$(ls -A $TMPDIR)" ];
then
    mv "$TMPDIR/"* "$TRASHDIR"
fi
CWD=$(pwd)

cargo build
EXE_PATH="$CWD/target/debug/git-sidequest"

(
    # Setup the test git repository for the no-args test
    WORKDIR=$(mktemp -d --tmpdir="$TMPDIR")
    cd "$WORKDIR" || return 1
    git init --quiet
    head --bytes=1000 /dev/urandom | base64 > file.txt
    git add file.txt
    git commit -m "Initial commit" --quiet
    git switch -c original-branch --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some content" --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some more content" --quiet

    # Run the test
    head --bytes=1000 /dev/urandom | base64 > non-sidequest.txt
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    head --bytes=1000 /dev/urandom | base64 > sidequest.txt
    git add sidequest.txt
    $EXE_PATH sidequest-branch --message "Commit message for sidequest"

    git log --graph --all --oneline
)

(
    # Setup the test git repository for the onto test
    WORKDIR=$(mktemp -d --tmpdir="$TMPDIR")
    cd "$WORKDIR" || return 1
    git init --quiet
    head --bytes=1000 /dev/urandom | base64 > file.txt
    git add file.txt
    git commit -m "Initial commit" --quiet
    git switch -c onto-branch --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some content" --quiet
    git switch -c original-branch --quiet
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some more content" --quiet

    # Run the test
    head --bytes=1000 /dev/urandom | base64 > non-sidequest.txt
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    head --bytes=1000 /dev/urandom | base64 > sidequest.txt
    git add sidequest.txt
    $EXE_PATH --onto onto-branch sidequest-branch -m "Commit message for onto-branch sidequest"

    git log --graph --all --oneline
)

cd "$CWD" || return 1
