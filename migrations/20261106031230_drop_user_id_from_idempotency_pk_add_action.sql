-- Hard cutover: idempotency keys are no longer scoped by user_id. A form-minted
-- UUIDv7 is unique on its own; `action` distinguishes save_draft vs publish so
-- one compose/edit form key can cover both buttons without colliding.
-- Existing rows are publish-path rows (the only writers before this change).
ALTER TABLE idempotency ADD COLUMN action TEXT NOT NULL DEFAULT 'publish';
ALTER TABLE idempotency DROP CONSTRAINT idempotency_pkey;
ALTER TABLE idempotency ADD PRIMARY KEY (idempotency_key, action);
ALTER TABLE idempotency ALTER COLUMN action DROP DEFAULT;
