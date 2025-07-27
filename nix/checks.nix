{
  craneLib,
  commonArgs,
  cargoArtifacts,
}: {
  fmt = craneLib.cargoFmt {
    inherit (commonArgs) src;
  };

  clippy = let
    clippyScope = "--lib --bins"; # we want this because tests require active db connection
  in
    craneLib.cargoClippy (commonArgs
      // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "${clippyScope} -- --deny warnings";
      });
}
