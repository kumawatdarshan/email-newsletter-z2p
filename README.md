# My learnings and some notes

## Tech Stack Differences
- Using **Axum** instead of `actix-web`.
- Sqlite since commit `b7fd3db`.
  - Migration was an experience...
  - I had never used sqlite, I wasn't aware the migration would prove to be this much work.
  - For example, the types: uuid, timestampz don't exist in sqlite.
  - But the biggest problem was actually caused by `20251025083651_make_status_not_null_in_subscriptions.sql`
    - Sqlite doesn't support nested txn.
    - Sqlite has an extremely limited `ALTER TABLE` support.
    - Only way I could find to hack it in is to *recreate* the table.
  - That sounds ridiculous for a system in production. Absolutely never change your db midway. 
  - Damm the problems didn't end at just queries. Bigger headache turned out to be how to handle tests.
    - Creating new `.db` is easy way out but that's just messy.
    - AsyncDrop trait is not yet here to do delete file afterwards.
    - In memory db has its own set of problems with it being not shared across the entire test.
      - Shared memory is an option which I ended up using. There are some caveats to it, I hope I don't run into them
  - I could have created a trait based dependency injection but I wanted to see how feasible it is to fully migrate. Plus that would have taken more time when my goal was to minimize the overhead of running tests
- Created a proc macro to derive 2 things, `IntoResponse` from axum for our error types and Debug Chain impl(that the book mentions). It happened to be my first macro I ever wrote. Code, imo, is very simple and can be used to look at how to write macros. 

- Using **Nix flakes** for the entirety of the deployment pipeline.
  - Also Integrating Nix into Github Actions.
- Brought back services flake for redis. I was doubting between looking for an embeddable db, thought `moka db` would be good enough for my use cases. Alas, was I wrong, while evaluating to choose `moka`, I pondered over the entire reason why I was setting up it now and not relying on the memory store for session. `PERSISTENCE`. (look this)[## 1. Axum-messages store] My tests had been failing because of persistence. 

### Minor Differences
1. Cargo Workspace is solely to reduce compile times, The project is not really large enough to require it but the compile times are just so much better now.
1. No need for serde-aux for `5.4`. Updates... ig?
1. Not deploying it. I am... *broke*.
1. `validator` Crate had massive API changes, Making it easy to validate directly on the struct and thus simplifying alot of the stuff the book manually implemented.
1. API changes in `fake` crate, the constraints for g is now stricter and requires a type that implements `Rng` trait which comes from `rand` crate.
1. Using `ReSend` for sending mails. Why?
> 3000/month instead of a 100/day.
> Doesn't require a professional mail. (primary reason why I used this)
1. Using `dotenvy` instead of risking API uploads to GitHub

# Side Questing

## 1. Axum-messages store

To my understanding, axum-messages rely on a external store impl. One such impl is provided by `tower-sessions` itself, the "MemoryStore". That seemed like the perfect option as the next step should be redis. But I quickly ran into a bad bad problem. 
All, literally all my tests failed. I couldn't figure out why. I thought its some library bug. I cloned `axum-messages`, I thought I found the bug. It exists in `tower-sessions`.
Turns out I was stupid in that regards, I forgot to keep `tower-sessions` and `tower-sessions-core` in sync which caused those bugs. Anyway, I again forked `tower-sessions` trying to find whats wrong.
And literally nothing. I had a moment of epiphany as I stared at the error message which was with me since the very beginning `*session not found in request extensions;* do tower-sessions versions match?`. I had not setup sessions. It had no way of retrieving, ofcourse it was failing. 

## 1. thiserror transparent attribute

This should not be used when we don't want to expose underlying error cause to the user. That is, leaking internal representation.
What are such situations? When mapping to DB/IO errors. Or simply any error that could help adversaries to sabotage our system.Cheat Code: wherever there is UnexpectedError, avoid transparent.

The book had used it in several occasions like in `ConfirmationError::UnexpectedError`, `SubscribeError::UnexpectedError` and finally `PublishError::UnexpectedError` from where I took notice of this.

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

1. Using **Juspay's service flake** to run PostgreSQL.
1. Using **process-compose** to `run` the PostgreSQL server.
1. When using `Crane` as builder, it will also run `cargo test`. This could turn problematic since we run integration test. To disable this behavior, use `doCheck = false;`
1. When Building docker img, you have to manually specify all runtime files. Use `pkgs.linkFarm`, otherwise it will be copied as a flat structure.


## Database Quirks

1. You must **delete the `data/` directory** if you’ve made any config changes like creating a role.  
   Gave me a lot of headache.

## Tower/Axum Quirks

1. To replicate what the book does at the end of Chapter 4 — having `request_id` in the logs:
  - You can create a closure.
  - Or, better yet, implement the trait `tower_http::trace::MakeSpan` for a type `T`.
  - Then pass it to `TraceLayer::make_span_with`.
> Done in commit `9cb3376` in `/src/telemetry.rs`, struct `RequestIdMakeSpan`.


## Notes for self.

### Contribution
1. See if i can improve sqlx error msg. I have a big gripe with it right now is that, it cannot differentiate if it needs active connection for a query or `SQLX_OFFLINE` mode would work. This gave me a lot of pain while debugging my `cargo clippy --all-targets`.


```bash
curl 'https://api.resend.com/emails' \
     -X POST \
     -H 'Accept: application/json' \
     -H 'Content-Type: application/json' \
     -H 'Authorization: Bearer re_xxxxxxxxx' \
     -d
    ```
```json
{
  // maybe a string or string[]
  "from": "Acme <onboarding@resend.dev>",
  "to": ["delivered@resend.dev"], 
  "subject": "hello world",
  "text": "Plain text version",
  "html": "<p>it works!</p>",
  "reply_to": "onboarding@resend.dev"
}
```

response: 
```json
{
  "id": "49a3999c-0ce1-4ea6-ab68-afcd6dc2e794"
}
```


### Acknowledgements:

1. [Zero2Prod in axum by Zercerium](https://github.com/Zercerium/zero2prod)
> Helped me alot, especially in second half of chapter 10.
