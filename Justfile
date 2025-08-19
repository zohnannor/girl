# Print this list
default:
    just --list

# Format code
fmt check="":
    cargo +nightly fmt --all {{ if check == "" { check } else { "-- --check" } }}

# Run clippy
clippy fix="" force="":
    cargo clippy --all-targets --workspace \
        {{ if fix == "" { fix } else { if force == "" { "--fix" } else { "--fix --allow-dirty" } } }}
    cargo clippy --all-targets --all-features --workspace \
        {{ if fix == "" { fix } else { if force == "" { "--fix" } else { "--fix --allow-dirty" } } }}

alias c := clippy
alias lint := clippy

# Run tests with specific toolchain
test toolchain="stable":
    cargo +{{ toolchain }} test --all-targets --workspace
    cargo +{{ toolchain }} test --all-targets --all-features --workspace
    cargo +{{ toolchain }} test --doc --all-features --workspace

alias t := test

# Build docs
doc no_deps="" private="" open="":
    RUSTDOCFLAGS="-Zunstable-options --default-theme=ayu --generate-link-to-definition --cfg docsrs" \
    cargo +nightly doc --workspace --keep-going \
        {{ if no_deps == "" { no_deps } else { "--no-deps" } }} \
        {{ if private == "" { private } else { "--document-private-items" } }} \
        {{ if open == "" { open } else { "--open" } }}
    RUSTDOCFLAGS="-Zunstable-options --default-theme=ayu --generate-link-to-definition --cfg docsrs" \
    cargo +nightly doc --workspace --all-features --keep-going \
        {{ if no_deps == "" { no_deps } else { "--no-deps" } }} \
        {{ if private == "" { private } else { "--document-private-items" } }} \
        {{ if open == "" { open } else { "--open" } }}

# Check MSRV
msrv:
    cargo +nightly update -Z minimal-versions
    cargo check --workspace
    cargo update

# Run all checks (~local CI)
ci:
    just fmt
    just clippy
    just test
    just doc
    just msrv
    @echo "All checks passed! :)"
