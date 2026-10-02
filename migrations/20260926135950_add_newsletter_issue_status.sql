-- Add migration script here

-- Track newsletter issue lifecycle separately from idempotency keys.
-- Idempotency remains forever (book-style); this enum is the source of truth
-- for whether an issue may still be enqueued for delivery.
-- This migration also converts published_at from TEXT to timestamptz (see the end).
CREATE TYPE newsletter.newsletter_issue_status AS ENUM (
    'draft',
    'queued',
    'sent',
    'failed'
);

-- The default lets ADD COLUMN succeed on a table that already has rows;
-- `draft` is the natural initial state for new issues.
ALTER TABLE newsletter.newsletter_issues
    ADD COLUMN status newsletter.newsletter_issue_status NOT NULL DEFAULT 'draft';

-- Existing rows were published before status existed; none of them is a real
-- draft. The production table is empty, so this only affects local/dev
-- databases (any with delivery still outstanding will show `sent` early).
UPDATE newsletter.newsletter_issues SET status = 'sent';

-- published_at was created as TEXT in the original newsletter_issues migration
-- (20231110183908). Existing values were all written by now(), so they are
-- well-formed timestamps with a UTC offset and cast cleanly. Converting gives the
-- column a real type: correct ordering/comparison, date maths, and validation.
-- If any row held a value that does not parse this migration fails and rolls
-- back (sqlx runs each migration in a transaction), leaving the table unchanged.
ALTER TABLE newsletter.newsletter_issues
    ALTER COLUMN published_at TYPE timestamptz USING published_at::timestamptz;
