-- Add migration script here
CREATE TABLE idempotency (
  user_id TEXT NOT NULL REFERENCES users (user_id),
  idempotency_key TEXT NOT NULL,
  response_status_code INTEGER,
  response_body BLOB,
  -- Store headers as a JSON array: [{"Content-Type", [97, 112, 112...]}]
  response_headers TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (user_id, idempotency_key)
);
