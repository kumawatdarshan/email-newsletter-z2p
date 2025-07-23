test-subcribe:
    #!/usr/bin/env fish
    function random_user
        set -l names arsha rudransh darshan vivaan aarya dev kartik reva meera yash
        set -l domains example.com test.io mail.dev devbox.lan

        set -l name (random choice $names)
        set -l domain (random choice $domains)
        set -l suffix (random | string sub -l 3)

        set -l email "$name$suffix@$domain"

        echo "name=$name&email=$email"
    end
    curlie -X POST http://localhost:8000/subscribe \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d (random_user)


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


