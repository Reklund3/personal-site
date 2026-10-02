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

ALTER TABLE newsletter.newsletter_issues
    ADD COLUMN status newsletter.newsletter_issue_status NOT NULL DEFAULT 'draft';

-- Existing rows were already published under the pre-status schema.
-- Prefer `queued` when delivery work is still outstanding so a mid-flight
-- issue is not marked `sent` while issue_delivery_queue rows remain; otherwise
-- `sent`. (This migration has not shipped to prod; edit in place rather than
-- add a follow-up backfill.)
UPDATE newsletter.newsletter_issues ni
SET status = CASE
    WHEN EXISTS (
        SELECT 1
        FROM newsletter.issue_delivery_queue q
        WHERE q.newsletter_issue_id = ni.newsletter_issue_id
    ) THEN 'queued'::newsletter.newsletter_issue_status
    ELSE 'sent'::newsletter.newsletter_issue_status
END;

-- published_at was created as TEXT in the original newsletter_issues migration
-- (20231110183908). Existing values were all written by now(), so they are
-- well-formed timestamps with a UTC offset and cast cleanly. Converting gives the
-- column a real type: correct ordering/comparison, date maths, and validation.
-- If any row held a value that does not parse this migration fails and rolls
-- back (sqlx runs each migration in a transaction), leaving the table unchanged.
ALTER TABLE newsletter.newsletter_issues
    ALTER COLUMN published_at TYPE timestamptz USING published_at::timestamptz;
