-- Add migration script here
INSERT INTO
  users (user_id, username, password_hash)
VALUES
  (
    '8084068e-4bc7-483c-8d9a-e2e7a02b956c',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$87LFZLF90HqnarKyBx4nxg$YZ9odNZxsfJMbImENdVqnktEzJHwrzBTd0iN0KkH20A'
  );
