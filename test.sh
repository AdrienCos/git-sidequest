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
WORKDIR=$(mktemp -d --tmpdir="$TMPDIR")
CWD=$(pwd)

cargo build
EXE_PATH="$CWD/target/debug/git-sidequest"

(
    # Setup the test git repository
    cd "$WORKDIR" || return 1
    git init
    head --bytes=1000 /dev/urandom | base64 > file.txt
    git add file.txt
    git commit -m "Initial commit"
    git switch -c original-branch
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some content"
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    git add file.txt
    git commit -m "Add some more content"

    # Run the test
    head --bytes=1000 /dev/urandom | base64 > non-sidequest.txt
    head --bytes=1000 /dev/urandom | base64 >> file.txt
    head --bytes=1000 /dev/urandom | base64 > sidequest.txt
    git add sidequest.txt
    $EXE_PATH sidequest-branch

    git log --graph --all --oneline
)

cd "$CWD" || return 1
