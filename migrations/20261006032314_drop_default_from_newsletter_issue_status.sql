-- Any code path that inserts into newsletter.newsletter_issues without an
-- explicit status now fails loudly (NOT NULL violation, safe to retry)
-- instead of silently defaulting to 'draft' -- the one status this app
-- will re-enqueue for delivery. Current application code always sets
-- status explicitly on every insert, so this is a no-op today; it only
-- closes the window during a mixed-version rolling deploy where an
-- older, status-unaware binary could otherwise mislabel an
-- already-delivered issue as still needing delivery.
ALTER TABLE newsletter.newsletter_issues
    ALTER COLUMN status DROP DEFAULT;
