-- Reorganize tables into bounded-context schemas.
-- See PLAN.md for the rationale.
--
-- `newsletter` schema: content distribution and subscriber management.
-- `audience` schema: personal-brand engagement (currently just contact inquiries).
-- `users` and `idempotency` remain in `public` — cross-cutting infrastructure
-- concerns, not part of a business domain. `_sqlx_migrations` also stays in
-- `public`; it is sqlx's own bookkeeping, not domain data.

CREATE SCHEMA newsletter;
CREATE SCHEMA audience;

-- Newsletter context
ALTER TABLE subscriptions SET SCHEMA newsletter;
ALTER TABLE subscription_tokens SET SCHEMA newsletter;
ALTER TABLE newsletter_issues SET SCHEMA newsletter;
ALTER TABLE issue_delivery_queue SET SCHEMA newsletter;

-- Audience context
ALTER TABLE contacts SET SCHEMA audience;
