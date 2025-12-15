-- Add migration script here
CREATE TABLE users(
    user_id TEXT NOT NULL PRIMARY KEY, -- that not null you see is sqlite's poor decision(bugs) 
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
) 
