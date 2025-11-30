{
  meta,
  pkgs,
  craneLib,
  commonArgs,
  cargoArtifacts,
}: let
  inherit (meta) version name;
  pname = name;
in rec {
  # YOU NEED TO RUN `cargo sqlx prepare -- --release` FOR THIS
  default = craneLib.buildPackage (commonArgs
    // {
      inherit version cargoArtifacts pname;
      inherit (commonArgs) buildInputs nativeBuildInputs;
      doCheck = false;
      RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C target-cpu=native";
      # SQLX_OFFLINE = true; was needed in pgsql, i dont think i need in sqlite
      CARGO_WORKSPACE_DIR = commonArgs.src;
    });

  docker = let
    bin = "${default}/bin/${pname}"; # this is still giving me a `result` directory
    runtimeDirs = [
      {
        name = "configuration";
        path = "${commonArgs.src}/configuration";
      }
      {
        name = "migrations";
        path = "${commonArgs.src}/migrations";
      }
    ];
    runtime = pkgs.linkFarm "config" runtimeDirs;
  in
    pkgs.dockerTools.buildLayeredImage {
      inherit name;
      tag = "latest";
      contents = [runtime];
      config = {
        Entrypoint = [bin];
        ExposedPorts."8000/tcp" = {};
      };
    };
}
