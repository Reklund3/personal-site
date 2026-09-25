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

impl NewsletterIssueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::Failed => "failed",
        }
    }
}
