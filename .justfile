req arg:
    curlie -v http://127.0.0.1:3000/{{arg}}

test:
    cargo test

migration:
    sqlx database create
    sqlx migrate run

nix-run:
    nix run \
      --option substitute true \
      --option substituters https://cache.nixos.org \
      --option trusted-public-keys cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=

