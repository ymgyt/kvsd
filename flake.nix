{
  description = "kvsd";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, fenix, crane, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-e4mlaJehWBymYxJGgnbuCObVlqMlQSilZ8FljG9zPHY=";
        };

        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.; # The original, unfiltered source
          filter = path: type:
            # Default filter from crane (allow .rs files)
            (craneLib.filterCargoSources path type);
        };

        darwinDeps = [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        commonArgs = {
          inherit src;
          strictDeps = true;

          pname = "kvsd-deps";
          version = "0.1";

          buildInputs = [ ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwinDeps;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        kvsdCrate =
          craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };
        kvsd = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          inherit (kvsdCrate) pname version;
        });

        # TODO: should parse .cargo/audit.toml
        ignoreAdvisories = pkgs.lib.concatStrings
          (pkgs.lib.strings.intersperse " "
            (map (x: "--ignore ${x}") [ "RUSTSEC-2023-0052" ]));

        checks = {
          inherit kvsd;

          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "-- --deny warnings";
          });

          nextest =
            craneLib.cargoNextest (commonArgs // { inherit cargoArtifacts; });

          audit = craneLib.cargoAudit {
            inherit src advisory-db;
            cargoAuditExtraArgs = "--ignore yanked ${ignoreAdvisories}";
          };

          fmt = craneLib.cargoFmt commonArgs;
        };

        ci_packages = with pkgs; [
          just
          nushell # just set nu as shell
          typos
        ];

        # Inherits from checks cargo-nextest, cargo-audit
        dev_packages = with pkgs;
          [ nixfmt git-cliff cargo-release oranda ] ++ ci_packages
          ## For cargo-release build
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwinDeps;

      in {
        inherit checks;

        packages.default = self.packages."${system}".kvsd;
        packages.kvsd = kvsd;

        apps.default = flake-utils.lib.mkApp {
          drv = kvsd;
          name = "kvsd";
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks
          checks = self.checks.${system};
          packages = dev_packages;
          shellHook = ''
            exec nu
          '';
        };

        devShells.ci = craneLib.devShell { packages = ci_packages; };
      });

  nixConfig = {
    extra-substituters = [ "https://kvsd.cachix.org" ];
    extra-trusted-public-keys =
      [ "Keykvsd.cachix.org-1:d4Vo1Qh1YC2H0kzCNapMJlP50J3JAydbP9cA+phQf/k=" ];
  };
}
