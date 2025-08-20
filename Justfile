# Print this list
default:
    just --list

cargo-hack := `which cargo-hack` # Requires cargo-hack to be installed

# Format code
fmt check="":
    cargo +nightly fmt --all {{ if check == "" { check } else { "-- --check" } }}

# Run clippy
clippy fix="" force="":
    cargo hack clippy --all-targets --feature-powerset --workspace \
        {{ if fix == "" { fix } else { if force == "" { "--fix" } else { "--fix --allow-dirty" } } }}

alias c := clippy
alias lint := clippy

# Run tests with specific toolchain
test toolchain="stable":
    cargo +{{ toolchain }} hack test --all-targets --feature-powerset --workspace
    cargo +{{ toolchain }} hack test --doc --all-features --workspace

alias t := test

# Build docs
doc no_deps="" private="" open="":
    RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -Zunstable-options --default-theme=ayu --generate-link-to-definition --cfg docsrs" \
    cargo +nightly hack doc --workspace --feature-powerset --features "document-features" \
        {{ if no_deps == "" { no_deps } else { "--no-deps" } }} \
        {{ if private == "" { private } else { "--document-private-items" } }} \
        {{ if open == "" { open } else { "--open" } }}

# Build docs for all features and open in default browser
doco:
    RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -Zunstable-options --default-theme=ayu --generate-link-to-definition --cfg docsrs" \
    cargo +nightly doc --workspace --all-features --no-deps --open

# Check MSRV
msrv:
    cargo +nightly update -Z minimal-versions
    cargo hack check --workspace --feature-powerset
    cargo update

# Run all checks (~local CI)
ci: fmt clippy test doc msrv
    @echo "All checks passed! :)"
