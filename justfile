set shell := ["nu", "-c"]

# List recipe
default:
  just --list

# Run check
check:
  nix flake check --all-systems --accept-flake-config

test *flags:
	cargo nextest run {{ flags }}

# Run integration test
integration:
	cargo nextest run --test integration_test --no-capture
 
# Run audit
audit:
    cargo audit
