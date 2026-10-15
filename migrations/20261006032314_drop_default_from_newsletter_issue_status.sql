-- Reverses the DEFAULT 'draft' that migration 20260926135950_add_newsletter_issue_status
-- set on newsletter.newsletter_issues.status. PR #33 kept that default on purpose.
-- Dropping it is a deliberate decision, not a restoration of lost work.
--
-- 'draft' is the only status that publishing moves to 'queued' and enqueues for
-- delivery. Without a default, an insert that omits status fails with a NOT NULL
-- violation (23502) instead of silently creating a publishable draft, so every
-- writer must state the status. All current inserts already do; no behaviour
-- changes today.
--
-- Rollback: a binary that does not set status cannot publish until the default
-- is restored with ALTER COLUMN status SET DEFAULT 'draft'.
ALTER TABLE newsletter.newsletter_issues
    ALTER COLUMN status DROP DEFAULT;
