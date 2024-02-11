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
  };

  outputs = { self, nixpkgs, fenix, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-e4mlaJehWBymYxJGgnbuCObVlqMlQSilZ8FljG9zPHY=";
        };

        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;
          strictDeps = true;

          # pname and version required, so set dummpy values
          pname = "kvsd";
          version = "0.1";

          buildInputs = [ ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        kvsdCrate =
          craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };
        kvsd = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          inherit (kvsdCrate) pname version;
        });

        checks = {
          inherit kvsd;

          clippy =
            craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });

          nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;

            # NEXTEST_PROFILE = "";
          });

          fmt = craneLib.cargoFmt commonArgs;
        };

        ci_packages = with pkgs; [
          just
          nushell # just set nu as shell
        ];

        dev_packages = with pkgs;
          [ cargo-nextest ] ++ ci_packages
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ ];
      in {
        inherit checks;

        packages.default = self.packages."${system}".kvsd;
        packages.kvsd = kvsd;

        apps.default = flake-utils.lib.mkApp { drv = kvsd; };

        devShells.default = craneLib.devShell {
          packages = dev_packages;

          shellHook = ''
            # Use nushell as default shell
            exec nu
          '';
        };
        devShells.ci = craneLib.devShell { packages = ci_packages; };
      });

  nixConfig = {
    extra-substituters = [ "https://kvsd.cachix.org" ];
    extra-trusted-public-keys =
      [ "kvsd.cachix.org-1:d4Vo1Qh1YC2H0kzCNapMJlP50J3JAydbP9cA+phQf/k=" ];
  };
}
