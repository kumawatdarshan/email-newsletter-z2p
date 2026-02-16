{
  craneLib,
  commonArgs,
  cargoArtifacts,
  formatter,
  gitHooksLib,
}: let
  # Define the pre-commit check within the checks file
  pre-commit-check = gitHooksLib.run {
    src = commonArgs.src;
    hooks = {
      treefmt.enable = true;
    };
    settings.treefmt.package = formatter;
  };
in
  {
    inherit formatter;
    inherit pre-commit-check;

    clippy = craneLib.cargoClippy (commonArgs
      // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--lib --bins -- -D warnings";
      });

    # Merge the shellHook from pre-commit into your checks
    # so 'nix flake check' validates the hooks themselves.
  }
  // pre-commit-check.checks
