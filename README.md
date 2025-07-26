# Things Different from Original Book

## Tech Stack Differences
- Using **Axum** instead of `actix-web`.
- Using **Nix flakes** for the entirety of the deployment pipeline.
  - Also Integrating Nix into Github Actions.

# Side Questing

## 1. Status Codes: `400 Bad Request` vs `422 Unprocessable Entity`

Missing data for content type `application/x-www-form-urlencoded` results in `UNPROCESSABLE_ENTITY` and not `BAD_REQUEST`.
~~Could be the case for all types of content types, I have yet to test. ~~ 

### Clarification:

- `400 Bad Request` — **Deserialization Error**
  - Server can't decode request into the expected type.
  - Common with: `Path`, `Query`
  - Example: invalid URL.

- `422 Unprocessable Entity` — **Validation Error**
  - Request format (`JSON`/`Form`) is valid, but contains incorrect data.
  - Common with: `Json`, `Form`
  - Example: missing required fields, wrong value types inside a valid JSON.

## 2. PostgreSQL: UNIQUE Constraint & Performance

- The `UNIQUE` constraint in PostgreSQL (and maybe all DBMS — unverified) introduces a **B-Tree index**.
- This index must be updated on every `INSERT` / `UPDATE` / `DELETE`.
- An area of optimization if I ever run into performance issues. Albeit rare I guess.

# Quirks

## Nix Quirks

1. Using **Juspay’s service flake** to run PostgreSQL.
2. Using **process-compose** to `run` the PostgreSQL server.

## Database Quirks

1. You must **delete the `data/` directory** if you’ve made any config changes like creating a role.  
   Gave me a lot of headache.

## Tower/Axum Quirks

1. To replicate what the book does at the end of Chapter 4 — having `request_id` in the logs:
  - You can create a closure.
  - Or, better yet, implement the trait `tower_http::trace::MakeSpan` for a type `T`.
  - Then pass it to `TraceLayer::make_span_with`.
> Done in commit `9cb3376` in `/src/telemetry.rs`, struct `RequestIdMakeSpan`.
