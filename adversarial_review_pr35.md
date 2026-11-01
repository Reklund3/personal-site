# Adversarial review: PR #35, Newsletter drafts: save, edit, list

- PR: https://github.com/Reklund3/personal-site/pull/35
- Branch: `newsletter-drafts`
- Reviewed head: `7e78303462717edff749285830a204501eaf5449`
- Review date: 2026-11-01

## Method and confidence

Two read-only reviewers covered the production code and design, and the test suite. No database was available (Postgres and Redis were unreachable), so no test was executed and no race interleaving was reproduced.

- **VERIFIED** covers only compile (`SQLX_OFFLINE=true cargo check --tests`), clippy, fmt, and the sqlx offline cache.
- **REASONED** means the finding was derived from reading the code. Each one needs a running Postgres to confirm.
- CI's Test job was green on this commit.

## Findings

### 1. Medium: publish can overwrite a saved draft with stale form content

Publish upserts whatever the publish form posted before it queues the issue (`src/routes/admin/newsletter/post.rs:89-102`, upsert at `post.rs:280-297`). The upsert has no version or `updated_at` check. The edit form posts its full content on Publish (`src/routes/admin/newsletter/get.rs:258-277`).

Two ways to lose a saved draft:

- **Stale tab.** Tab A loads draft v1. Tab B saves v2. Tab A then publishes. v1 is queued and sent to every confirmed subscriber, v2 is gone, and nothing warns the admin.
- **Concurrent save and publish.** A draft save and a publish of the same id interleave. The draft commits and shows "Draft saved." (true at commit time). Publish then overwrites the draft and queues its own content.

Once queued, the content cannot be recalled. No test catches this. The save-edit-publish test publishes the same content it last saved.

Status: REASONED.

### 2. Test-suite gaps

Status for every item in this section: REASONED (no tests were executed).

- **High: edit-form wiring is not asserted.** The hidden `newsletter_issue_id` and `idempotency_key` fields on the edit form are never checked. The reviewer read the wiring as correct: the edit form reuses the path id (`get.rs:150-207`). Replacing that id with `Uuid::new_v4()` would still pass every test. The new-form test only checks that the `name="newsletter_issue_id"` attribute exists.
- **High: the save-edit-publish test does not isolate the edit step.** `tests/api/newsletter.rs:1432-1513`. Publish re-sends the edited content, so the test passes even if the draft save wrote nothing. "Delivered content is edited content" is only shown for the publish path.
- **Medium: the concurrent save and publish test does not establish the race property.** `tests/api/newsletter.rs:1784-1842`.
  - `tokio::join!` forces no ordering. In the usual ordering, publish responds last, so the "Draft saved." assertion passes without exercising the guard.
  - Both requests share one cookie jar, which keeps only the last `_flash` cookie. If the draft responds last, the assertion fails even though the code is correct. The test is order-dependent and can flake.
  - No test would detect removal of `FOR UPDATE` from the draft handler.
  - Severity was disputed between reviewers: the code reviewer rated it Low, the test reviewer rated it High. Rated Medium here.
- **Medium: escaping tests have gaps.** `tests/api/newsletter.rs:1690-1735`. The payloads contain no `&` and no `'`, so removing the `&` replacement from `escape_attr` would still pass. The flash sink has no hostile-input test. The recent-issues list sink is never exercised. The current code reads as correct: `&` is escaped first and every attribute is double-quoted.
- **Medium: the idempotency-row test passes if the draft save fails outright.** `tests/api/newsletter.rs:1737-1761`. It does not assert the POST status, and it has no positive control. A 400 or 500 leaves the row count at 0, so the test passes.
- **Medium: the list test cannot tell the Drafts section from Recent issues.** `tests/api/newsletter.rs:1620-1639`. Removing or inverting the draft filter would still pass. No test covers the recent-issues rendering.
- **Low: smaller gaps.**
  - The rejection test does not check `html_content`.
  - The malformed-path test accepts 404, so it cannot tell an unrouted path from a handled one.
  - A malformed `newsletter_issue_id` in a POST body is untested.
  - The helper `get_newsletter_edit_html` does not assert the status code.

### 3. Low: leading newlines are lost on each edit round trip

`src/routes/admin/newsletter/get.rs:268` and `get.rs:277` place content directly after the `<textarea>` start tag. The HTML parser drops a leading LF after `<textarea>` and `<pre>`. Each Save or Publish cycle therefore removes one leading newline, and the published content then differs from the stored draft.

Status: REASONED from the parse5 source. Not run in a browser.

### 4. Low: drafts with an empty title cannot be reached from the list

`get.rs:83` renders an `<a>` element with no link text, so the draft has no visible link in the list. Draft titles are not validated. Publish has the same missing validation.

Status: REASONED.

### 5. Low: CLAUDE.md does not match the code

`CLAUDE.md:117-127`.

- It describes Publish only as "draft→queued + enqueue". It omits that Publish first upserts the posted content, which is the behavior behind Finding 1.
- It says the edit form mints a `newsletter_issue_id`. The edit form reuses the id from the path.

Status: VERIFIED by reading the diff against the code.

## Verified safe

Checked by the code reviewer. Status: REASONED unless marked.

- **Concurrent first saves on a new id.** `INSERT ... ON CONFLICT` serializes them. There is no unique-violation 500. The last writer wins, which is Finding 1.
- **Publish landing between the draft's lock and its upsert.** The re-check rolls back, so a non-draft row never shows a false "Draft saved." flash.
- **Two tabs with different idempotency keys on the same id.** One request gets Queued and the other gets AlreadyHandled. No double enqueue.
- **Non-draft content.** Every write path is guarded against non-draft rows.
- **Escaping.** Attributes are double-quoted and `&` is escaped first. Textarea, `<pre>`, and flash output are escaped. Nothing is reflected from the query string.
- **Routes and auth.** All four new routes sit inside the admin authentication scope. The SPA routing contract is untouched.
- **Worker.** Drafts are never enqueued, dequeued, or marked `sent`.
- **Validation.** Draft save and publish are equally loose: neither applies domain newtypes to title or body. The 16 KB form limit applies to both.
- **sqlx cache.** 56 files, no orphans. Status: VERIFIED.

## Not checked

- `cargo test` was not run. The PR's claim of "19 lib + 76 api passed" is unconfirmed by these reviewers, although CI's Test job is green on this commit.
- The race interleavings were not reproduced.
- `cargo sqlx prepare --check` needs a database.
- Everything marked REASONED needs a running Postgres to confirm.
