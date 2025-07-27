{
  meta,
  pkgs,
  craneLib,
  commonArgs,
  cargoArtifacts,
}: let
  inherit (meta) name version;
in rec {
  default = craneLib.buildPackage (commonArgs
    // {
      inherit version cargoArtifacts;
      inherit (commonArgs) buildInputs nativeBuildInputs;
      doCheck = false;
      pname = name;
      RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C target-cpu=native";
    });

  docker = let
    bin = "${default}/bin/${name}";
    runtimeDirs = [
      {
        name = "configuration";
        path = ../configuration;
      }
      {
        name = "migrations";
        path = ../migrations;
      }
    ];
    runtime = pkgs.linkFarm "config" runtimeDirs;
  in
    pkgs.dockerTools.buildLayeredImage {
      inherit name;
      tag = "v${version}";
      contents = [runtime];
      config = {
        Entrypoint = [bin];
        ExposedPorts."8000/tcp" = {};
      };
    };
}
