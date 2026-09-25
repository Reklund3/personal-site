-- Track newsletter issue lifecycle separately from idempotency keys.
-- Idempotency remains forever (book-style); this enum is the source of truth
-- for whether an issue may still be enqueued for delivery.
CREATE TYPE newsletter.newsletter_issue_status AS ENUM (
    'draft',
    'queued',
    'sent',
    'failed'
);

ALTER TABLE newsletter.newsletter_issues
    ADD COLUMN status newsletter.newsletter_issue_status NOT NULL DEFAULT 'draft';

-- Existing rows were already published under the pre-status schema.
UPDATE newsletter.newsletter_issues
SET status = 'sent';
