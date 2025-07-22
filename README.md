# Things different from original book.
- Using Axum instead of actix-web.
- Using nix flakes for entirety of deployment pipeline.

### Side Questing

~~1. Missing data ~~for content type `application/x-www-form-urlencoded`~~ results in `UNPROCESSABLE_ENTITY` and not `BAD_REQUEST`. ~~Could be the case for all types of content type, i have yet to test.~~ It is indeed the case with all types of content types.~~
1. Okay, so, 422 is for `Validation error` and 400 is for `Deserialization error`. Wait what?

+ `400 Bad Request` — **Deserialization error**
  >- Server can't decode request into the expected type.
  >- Common with: `Path`, `Query`
  >- Example: invalid URL.

+ `422 Unprocessable Entity` — **Validation error**
  >- Request format (JSON/form) is valid, but incorrect data.
  >- Common with: `Json`, `Form`
  >- Example: missing required fields, wrong value types inside a valid JSON.

2. UNIQUE constraint in postgres (and maybe all dbms, unchecked info) introduces B-Tree index which needs to be updated on every `INSERT`/`UPDATE`/`DELETE` query. An area of optimization if I ever run into perf issue. 


## Quirks

### Nix Quirks
1. Using juspay's service flake to use postgres.
2. Using process-compose to `run` psql server.

### Database Quirks
1. must delete `data/` dir if made any changes to config like creating a role. Gave me a lot of headache
