{
  description = "Email Newsletter Service, guided project from Zero 2 Production, implemented in axum instead.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
    crane,
    process-compose-flake,
    services-flake,
    treefmt-nix,
    ...
  }: let
    config = builtins.fromJSON (builtins.readFile ./configuration/base.json);
    meta =
      (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.metadata.crane;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [fenix.overlays.default];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      craneLib = crane.mkLib pkgs;

      src = pkgs.lib.fileset.toSource {
        root = ./.;
        fileset = pkgs.lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
          ./migrations
          ./configuration
          ./.sqlx
        ];
      };

      # runtime deps
      buildInputs = [];
      # Build deps
      nativeBuildInputs = with pkgs; [
        sqlx-cli
        mold
        sccache
      ];
      commonArgs = {
        inherit src buildInputs nativeBuildInputs;
        strictDeps = true;
        SCCACHE_DIR = "/tmp/sccache"; # not using docker for dev, fine with cache miss.
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      treefmt = treefmt-nix.lib.evalModule pkgs {
        projectRootFile = "flake.nix";
        programs = {
          alejandra.enable = true;
          taplo.enable = true;
          rustfmt.enable = true;
          jsonfmt.enable = true;
          just.enable = true;
        };
      };

      formatter = treefmt.config.build.wrapper;
    in {
      inherit formatter;
      packages = import ./nix/packages.nix {
        inherit meta pkgs craneLib commonArgs cargoArtifacts;
      };

      checks = import ./nix/checks.nix {
        inherit
          craneLib
          commonArgs
          cargoArtifacts
          formatter
          ;
      };

      devShells = import ./nix/devshell.nix {
        inherit config pkgs;
        inherit (commonArgs) buildInputs nativeBuildInputs;
        inherit process-compose-flake services-flake;
      };
    });
}
