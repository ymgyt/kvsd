# set shell := ["nu", "-c"]

# List recipe
default:
  just --list

# Run check
check:
  nix flake check --all-systems --accept-flake-config

# Run integration test
integration:
	cargo nextest run integration --test integration_test --no-capture
 
