{
  craneLib,
  commonArgs,
  cargoArtifacts,
  git-hooks-bin,
  self,
}: let
  pre-commit-check = git-hooks-bin.run {
    src = self;
    hooks = {
      alejandra.enable = true;
      taplo.enable = true;
    };
  };
in {
  inherit pre-commit-check;
  fmt = craneLib.cargoFmt {
    inherit (commonArgs) src;
  };

  # clippy = let
  #   clippyScope = "--lib --bins"; # we want this because tests require active db connection
  # in
  #   craneLib.cargoClippy (commonArgs
  #     // {
  #       inherit cargoArtifacts;
  #       cargoClippyExtraArgs = "${clippyScope} -- --deny warnings";
  #     });
}
