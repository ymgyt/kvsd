set shell := ["nu", "-c"]

arch := arch()
os := if os() == "macos" { "darwin" } else { "linux" }

# List recipe
default:
  just --list

# Run check
check:
  nix flake check --all-systems --accept-flake-config

# Run clippy
lint: 
	cargo clippy --all-features --tests --benches

test *flags:
	cargo nextest run {{ flags }}

# Run integration test
integration:
	cargo nextest run --test integration_test --no-capture
 
# Run audit
audit:
    nix build .#checks.{{arch}}-{{os}}.audit --print-build-logs 

changelog *flags:
	git cliff out> CHANGELOG.md

release *flags: changelog
	cargo release {{ flags }}
