-- Add migration script here

UPDATE subscriptions
SET status = 'confirmed'
WHERE status IS NULL;
