# Default recipe
default:
    just --list


# Format code
fmt check="":
    cargo +nightly fmt --all {{ if check == "" { check } else { "-- --check" } }}

# Run clippy
clippy:
    cargo clippy --all-targets --all-features --workspace
alias c := clippy

# Run tests with specific toolchain
test toolchain="stable":
    cargo +{{ toolchain }} test --all-targets --all-features
alias t := test

# Build docs
doc:
    cargo +nightly doc --all-features --no-deps --document-private-items

# Check MSRV
msrv:
    cargo +nightly update -Z minimal-versions # is this correct?
    cargo check --workspace
    cargo update

# Run all checks (~local CI)
ci:
    just fmt
    just clippy
    just test
    just doc
    just msrv
