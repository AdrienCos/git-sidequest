[private]
@default:
    just --list

# Lint the code
check:
    cargo check
# Build the dev version of the app
build:
    cargo build
# Build the release version of the app
release:
    cargo build --release
# Build and install the app to $HOME/.cargo/bin
install:
    cargo install --path .
# Remove build artifacts
clean:
    cargo clean
    rm -f result
