[private]
@default:
    just --list

# Lint the code
check:
    cargo check
# Build the dev version of the app
build:
    cargo build
# Run the test suite
test:
    bats ./tests/*
# Build the release version of the app
release:
    cargo build --release
# Build and install the app to $HOME/.cargo/bin
install:
    cargo install --path .
# Update the package dependencies
update-deps:
    cargo update
# Update the flake inputs
update-flake:
    nix flake update
# Enter the DevShell of the project, and add it to the GCroots
activate:
    nix develop --profile ./profile --command zsh
# Remove build artifacts
clean:
    cargo clean
    rm -f result
    rm -f profile
    rm -f profile-*
