{
  description = "Email Newsletter Service, guided project from Zero 2 Production, implemented in axum instead.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = inputs: let
    inherit (inputs) self nixpkgs fenix flake-utils crane;
    config = builtins.fromJSON (builtins.readFile ./configuration/base.json);
    meta = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [fenix.overlays.default];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      rustToolchain = pkgs.fenix.minimal.withComponents [
        "cargo"
        "clippy"
        "rust-src"
        "rustc"
        "rustfmt"
        "rust-analyzer"
      ];

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

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
      buildInputs = with pkgs; [
        openssl
      ];
      # Build deps
      nativeBuildInputs = let
        isLinux = pkgs.lib.optionals pkgs.stdenv.isLinux;
      in
        with pkgs;
          [
            pkg-config
            sqlx-cli
          ]
          ++ isLinux [
            mold
          ];

      commonArgs = {
        inherit src buildInputs nativeBuildInputs;
        # SQLX_OFFLINE = true;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in {
      packages = import ./nix/packages.nix {
        inherit meta pkgs craneLib commonArgs cargoArtifacts;
      };

      checks = import ./nix/checks.nix {
        inherit craneLib commonArgs cargoArtifacts;
      };

      devShells = import ./nix/devshell.nix {
        inherit config pkgs;
        inherit (commonArgs) buildInputs nativeBuildInputs;
        inherit (inputs) process-compose-flake services-flake;
      };
    });
}
