/// Lifecycle of a `newsletter.newsletter_issues` row.
///
/// Mapped from the Postgres enum `newsletter.newsletter_issue_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(
    type_name = "newsletter.newsletter_issue_status",
    rename_all = "snake_case"
)]
pub enum NewsletterIssueStatus {
    Draft,
    Queued,
    Sent,
    Failed,
}
