# PR #33 review findings

Adversarial review of PR #33, "Newsletter issue status: draft | queued | sent | failed", at head
`0df6406` (branch `expire-idempotency-keys`), 2026-09-27. Two independent reviews were merged:
a manual one and a `/code-review` agent. Every agent claim was checked against the code before
being included.

The first review round covered the earlier idempotency-TTL version of this PR. The branch has
since been rewritten, so those findings no longer apply. The one that did (the CI failure caused by
`handle.abort()` inside the loop in `tests/api/subscriptions.rs`) is fixed in `6d99dd4`.

## Summary

- **No correctness bug in code the app can actually run.** The worker locking works under
  Postgres's default READ COMMITTED isolation, which nothing in the code or config overrides.
  The publisher and workers can't deadlock, and replays behave correctly under concurrent
  submissions.
- **Two decisions to make:** what `sent` and `failed` mean (item 1), and whether to keep the
  draft-accept path (item 2). Most of the other fixes follow from those two.
- **Verified:** CI is fully green (61 tests, including all 19 `newsletter::` tests).
  `SQLX_OFFLINE=true cargo check --tests` and `cargo clippy -- -D warnings` pass locally.
- **Not run locally:** the integration tests (Docker wasn't running), so test results come from
  the CI logs.

## Findings (most severe first)

### 1. `sent` doesn't mean delivered, and `failed` is never written

- The PR description says `failed` is set when an issue can't be loaded. The code says the
  opposite: `src/issue_delivery_worker.rs:64` calls `failed` "reserved for future use", and
  nothing ever sets it.
- `sent` is set whenever the issue's queue empties, even if every email failed.
  `email_5xx_does_not_mark_issue_failed` (`tests/api/newsletter.rs:808`) asserts `Sent` after
  the only send returned a 500.
- So after a Postmark outage every issue shows `sent`, and nothing lets you send it again.

### 2. The draft-accept path can't be reached (a design call, not a bug)

- No form sends `newsletter_issue_id`: `src/routes/admin/newsletter/get.rs` renders only the
  idempotency key.
- A draft is never saved, because the insert and the queue step run in one transaction.
  `AlreadyHandled` is only reachable by hand-crafting a POST with the id of an issue that's
  already been handled.
- This path alone justifies:
  - the `x-newsletter-queue-outcome` header stored with saved idempotency responses (`post.rs:15`)
  - its attach, strip and read helpers
  - the `pub` exports in `src/routes/admin/newsletter/mod.rs:5`
  - the 400 error path
  - about eight tests
- It has its own defects (found by the agent, verified):
  - **Requires content it ignores.** `title`, `text_content` and `html_content` are required
    `String` fields (`post.rs:19`). A POST with just `idempotency_key` and `newsletter_issue_id`
    is rejected with a 400 before the handler runs, which is why every test sends dummy
    content.
  - **Wrong publish time.** `published_at` is set when the draft is inserted and never updated
    when it moves to `queued`. A draft accepted days later records its creation time.
  - **Five statements where two would do** (`post.rs:210`). The `draft` status is overwritten
    in the same transaction, so nobody ever sees it.
  - **Misleading replay message.** Replaying a publish saved before this deploy (no outcome
    header) shows "already been handled" instead of success (`post.rs:147`). This is
    deliberate and the message is still true, so it's cosmetic.

### 3. The worker's missing-issue branch is dead code

- `src/issue_delivery_worker.rs:99` and `drain_orphaned_delivery_tasks` (`:199`) can never run.
  The queue's foreign key has no `ON DELETE CASCADE`, so an issue can't be deleted while queue
  rows point at it.
- Both tests that reach this branch switch off foreign-key checks with
  `SET session_replication_role = 'replica'` (`tests/api/newsletter.rs:558`, `:757`). That needs
  a Postgres superuser, so those tests fail if `DATABASE_URL` uses a normal user.
- `idempotent_replay_with_deleted_issue_flashes_already_handled` (`:517`) can't fail because of
  the deletion it sets up. Replay reads the stored header and never looks at the issue.
- The drain would also deadlock if two workers hit it at once (found by the agent). Each
  worker's `DELETE ... WHERE newsletter_issue_id = $1` waits on the row the other one holds.

### 4. The race test may not actually race

`concurrent_task_completions_flip_issue_to_sent_exactly_once` (`:1018`) runs both workers with
`tokio::join!` inside one task. Whether they really overlap depends on timing, so removing the
`FOR UPDATE` lock in `delete_task` (`issue_delivery_worker.rs:165`) would probably still pass. It
also never checks the "exactly once" in its name. Not verified by running it.

### 5. The migration was renamed and then edited in place

`20260925030100` was renamed to `20260926135950` (`b142ed2`) and then edited again (`ca44f09`).
If a local or review database ran either earlier version, `sqlx migrate run` will refuse to
continue until the database is reset. Production is only safe if this branch was never deployed
there.

### 6. Issues can get stuck in `queued` during a rolling deploy (low)

This depends on how production applies migrations, which wasn't checked. Between running the
migrations and starting the new binary:

- The old binary's publish fails, because the default was dropped (intended, per the migration
  comment).
- An old worker empties queues without ever setting `sent`, so those issues stay `queued` for
  good. Nothing later reconciles them.

### 7. Nits

- `sqlx::Error::RowNotFound` is used to mean "issue doesn't exist" (`post.rs:95`, `:202`). A
  future `fetch_one` inside that function would turn a real database error into a misleading
  400.
- The `draft` → `queued` update ignores `rows_affected` (`post.rs:210`). It's safe only because
  of the `FOR UPDATE` lock just before it.
- Route internals are made `pub` just so tests can reach them.
- The "delivery is complete" query is copied into both `post.rs:223` and
  `issue_delivery_worker.rs:175`.
- Every delivery task now also locks the issue row and runs an extra update before it commits
  (`issue_delivery_worker.rs:165`). Workers on the same issue take turns, and there's slightly
  more room for a failure after the send, which would email that subscriber again.

### 8. Docs are out of date

- `CLAUDE.md:123` still says expiring idempotency keys is listed in the README. The README no
  longer lists it.
- The Database section in `CLAUDE.md` doesn't mention the status enum.
- The PR description:
  - claims `failed` is set (it isn't)
  - says 9/9 newsletter tests pass (CI runs 19)
  - still has the smoke-test box unchecked

### Where the two reviews disagreed on severity

- The agent called the replay message a real defect. It's rated cosmetic here, because the
  fallback is deliberate and "already handled" is accurate.
- The agent listed the drain deadlock among the real defects. It's in code that can't run while
  the foreign key is enforced.

## Proposed fixes

### 1. Status meaning

**Recommended:** remove `failed` from the enum for now.

- Add it back with the retry work in issue #32, when "failed" can mean "still failing after N
  retries".
- Adding an enum value later is one line (`ALTER TYPE ... ADD VALUE`). Removing one later means
  recreating the type.
- Document `sent` as "every subscriber was attempted; API errors are logged, not tracked".
- Fix the PR description.

**Alternative:** add a `failed_deliveries int NOT NULL DEFAULT 0` column and increment it when a
send errors. When the queue empties, set
`status = CASE WHEN failed_deliveries = 0 THEN 'sent' ELSE 'failed' END`.

### 2. The draft-accept path

**Recommended:** cut it until there's a UI for drafts. A new publish becomes:

```rust
let issue_id = insert_newsletter_issue(&mut transaction, ...)   // INSERT ... status = 'queued'
    .await.context("Failed to store newsletter issue details").map_err(e500)?;
let enqueued = enqueue_delivery_tasks(&mut transaction, issue_id) // return rows_affected
    .await.context("Failed to enqueue delivery tasks").map_err(e500)?;
if enqueued == 0 {
    mark_issue_sent(&mut transaction, issue_id).await.map_err(e500)?; // no subscribers
}
```

The replay branch goes back to what `main` does: `success_message().send(); return Ok(saved_response);`

This removes:

- `newsletter_issue_id`, `QueueOutcome`, `already_handled_message`, and the outcome header with
  its helpers
- the `RowNotFound` → 400 mapping and the `pub` exports
- the extra statements per publish
- the unused `draft` enum value (add it back with the draft UI)
- these tests:
  - `already_queued_issue_is_not_enqueued_twice`
  - `publish_of_already_handled_issue_flashes_and_does_not_enqueue`
  - all three `idempotent_replay_*` tests
  - `http_accept_of_existing_draft_via_newsletter_issue_id`
  - `unknown_newsletter_issue_id_returns_400_and_rolls_back_idempotency`
  - `concurrent_queue_issue_for_delivery_enqueues_exactly_once`

**If kept, it needs:**

- `title`, `text_content` and `html_content` made optional, and required only when there's no
  issue id
- `published_at = now()` set on the `draft` → `queued` update
- a `QueueOutcome::NotFound` variant instead of borrowing `RowNotFound`
- a check of `rows_affected()` on the `draft` → `queued` update

### 3. The dead missing-issue branch

- Revert `get_issue` to `fetch_one`.
- Delete the `None` branch, `drain_orphaned_delivery_tasks`, the long comment above them, and
  `missing_issue_drains_orphaned_delivery_queue`.
- If the impossible ever happens, the worker errors loudly and retries, as it does on `main`.

This removes every use of `session_replication_role` and the deadlock.

### 4 and 6. The race test and issues stuck in `queued`

**Recommended:** replace the check after every task with a cleanup that runs when the queue is
empty. Put it inside `try_execute_task`, so the test helper `dispatch_all_pending_emails`
triggers it too:

```rust
let Some((transaction, issue_id, email)) = dequeue_task(pool).await? else {
    mark_drained_issues_sent(pool).await?;
    return Ok(ExecutionOutcome::EmptyQueue);
};
```

```sql
UPDATE newsletter.newsletter_issues ni
SET status = 'sent'
WHERE status = 'queued'
  AND NOT EXISTS (SELECT 1 FROM newsletter.issue_delivery_queue q
                  WHERE q.newsletter_issue_id = ni.newsletter_issue_id)
```

- **It can't race.** It only reads committed state, so two workers running it at once just
  repeat the same harmless update. The lock and the race test are no longer needed.
- **It repairs itself.** Issues left `queued` by an old worker during a rolling deploy get fixed
  on the next idle check.
- **`delete_task` goes back to delete-and-commit.** Workers no longer take turns on the issue
  row, and the window for a duplicate email shrinks back to what `main` has.
- **One copy of the query instead of two.**
- **Tradeoff:** `sent` only appears once the worker has nothing left to do, so there's up to
  about 10 seconds of delay after a queue empties, longer while other issues are still sending.

**If the check after every task is kept instead:**

- Move it into a `mark_sent_if_drained` helper so there's one copy.
- Replace the `tokio::join!` race test with one that always behaves the same way:
  1. In transaction 1, delete one of the two queue rows and call the helper.
  2. From another connection, run `SELECT ... FOR UPDATE NOWAIT` on the issue row and assert it
     fails with lock_not_available (55P03). That proves the lock is held until commit, and the
     test fails if someone removes it.
  3. Commit transaction 1, then delete the second row and call the helper, and assert `sent`.

### 5. The migration

The migration hasn't shipped, so make one final in-place edit that folds both migrations into
one and never has a default:

1. Add `status` as a nullable column.
2. Fill it in (`queued` if the issue still has queue rows, otherwise `sent`).
3. Set it `NOT NULL`.

Then reset the local database once
(`sqlx database drop && sqlx database create && sqlx migrate run`). From then on, add new
migrations instead of editing ones that have already run.

### 7 and 8. Nits and docs

- **Nits:** fixes 2 and 3 already remove the `RowNotFound` misuse, the ignored `rows_affected`,
  the `pub` exports and the old-replay message.
- **`CLAUDE.md:123`:** remove "expire idempotency keys" from Known Improvements. Keys are kept
  forever on purpose now.
- **`CLAUDE.md` Database section:** add the status column and when `sent` is set.
- **README:** the new line under "Improvements" describes current design rather than an
  improvement. Move it or drop it.
- **PR description:**
  - correct the `failed` claim
  - replace the test count with whatever CI reports after these changes
  - either run the smoke test or remove its checkbox

### Net effect

With the recommended fixes, the PR comes down to:

- a `queued | sent` enum and column
- a publish that sets it
- one idle-time cleanup in the worker

Most of the 1,052 added lines in `tests/api/newsletter.rs` go with the removed code.
