set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

export DAWN_ENV := "development"

# List the available commands
default:
    @just --list

# Run the server and two clients
dev clients="2":
    cargo xtask dev --clients {{clients}}

# Run the server on its own
server:
    cargo run -p server

# Run one client, for example: just client 3
client id="1":
    cargo run -p client -- --client-id {{id}}

# Run one game module's sandbox, for example: just sandbox movement
sandbox name:
    cargo run -p {{name}} --example sandbox --all-features --features sandbox_kit/dev

# Show art under the game's real mood, for example: just lookdev --capture ../shots
lookdev *args:
    cargo run -p client --example lookdev -- {{args}}

# Run every test in crates and libs
test:
    cargo nextest run --workspace --exclude xtask

# Run functions and modules in isolation
test-unit:
    cargo nextest run --workspace --exclude xtask --lib --bins

# Run the real server or client against a fake other side
test-integration:
    cargo nextest run --workspace --test integration

# Run the real server and clients together
test-e2e:
    cargo nextest run --workspace --test e2e

# Fail if unit and integration tests cover under 90% of changed lines
coverage *args:
    cargo xtask coverage {{args}}

# Run every check that CI runs, except the tests
check:
    cargo fmt --all --check
    cargo xtask check-architecture
    cargo xtask check-app-sides
    cargo xtask check-tests
    cargo clippy --workspace --all-targets -- -D warnings
    cargo xtask check-isolated

# Format all code
fmt:
    cargo fmt --all

# Show whether a newer Bevy is published and our dependencies support it
upgrade-status:
    cargo xtask upgrade-status
