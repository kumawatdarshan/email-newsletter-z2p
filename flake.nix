{
  description = "Email Newsletter Service, guided project from Zero 2 Production, implemented in axum instead.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , nixpkgs
    , rust-overlay
    }:
    let
      system = "x86_64-linux";
      overlays = [ rust-overlay.overlays.default ];
      rustFlags = "-C link-arg=-fuse-ld=mold -C target-cpu=native";

      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs;[
          (rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "clippy" ];
          })
          pkg-config
          openssl
        ];
        nativeBuildInputs = with pkgs;[
          mold
          just
          curlie
          cargo-watch
          cargo-expand
          rust-bin.stable.latest.rust-analyzer
          rust-bin.stable.latest.rustfmt
        ];

        shellHook = ''
          export RUST_BACKTRACE="1"
          export RUSTFLAGS="''${RUSTFLAGS:-""} ${rustFlags}" # prepend RUSTFLAGS if set from outside
        '';
      };
    };
}
