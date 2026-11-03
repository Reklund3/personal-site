# PR #35 (newsletter drafts): remaining requirements

- PR: https://github.com/Reklund3/personal-site/pull/35
- Branch: `newsletter-drafts`
- Verified at HEAD: `7265038`
- Date: 2026-11-03
- Source: verification of the earlier adversarial review (written against `7e78303`) against HEAD. The review file itself was removed from the branch in `bbc329a`; it is still in history at `bc6091d`.

## How to read this file

Each item states a requirement: a condition that must hold, and what evidence would show it holds. The mechanism is deliberately left open. Requirements marked "must fail" describe a test-suite property: a regression of the named kind must make a test fail.

Status labels:

- **PARTIAL**: some of the original finding is closed; the requirement below covers the rest.
- **NOT RESOLVED**: the original finding is still open as written.
- **RESIDUAL**: a related gap that the commits did not address, or that was introduced by them.

Item numbers 2a to 2g are inferred from the commit messages (`9abcbcb` cites 2a/2b/2e/2f; `7265038` cites 2c/2d/2g). Each item is matched by its content, not by its label.

Basis: all findings below are REASONED from the code unless marked otherwise. CI was green on `7265038` (22 lib tests and 83 api tests passed). No mutation runs or local `cargo test` have been done against these requirements.

## Partially resolved

### R1. Edit-page idempotency key must be checked by value (Finding 2a, PARTIAL)

- The edit page must render a publish idempotency key that is minted per page load and is a valid, non-constant key.
- The suite must fail if the rendered key is a constant (for example the nil UUID), or if two loads of the edit page render the same key.
- Current state: the test at `tests/api/newsletter.rs:1649-1652` checks only that `name="idempotency_key"` exists. It does not check the value or which form it sits in. `edit_newsletter_form` at `src/routes/admin/newsletter/get.rs:215-218` could render a constant for the edit draft without any test failing.

### R2. Concurrent save and publish must exercise the guards it names (Finding 2c, PARTIAL)

- **R2a.** The test must force the interleaving it claims to test. The current `Barrier` at `tests/api/newsletter.rs:1983-2013` aligns start times only. It does not force any ordering.
- **R2b.** The suite must fail when the post-upsert status re-check in `src/routes/admin/newsletter/post.rs:351-374` is removed, for the no-row race: compose Publish inserts a fresh id between a first save's empty `FOR UPDATE` and its upsert. Current state: no test covers this. The seeded-row test at HEAD does not reach the re-check. The 7e78303 test covered this case and was rewritten.
- **R2c.** The suite must fail when `FOR UPDATE` is removed from the draft save's first SELECT (`post.rs:311`), deterministically rather than in a fraction of iterations. Removing `FOR UPDATE` alone is currently behaviorally covered by the upsert's `WHERE status='draft'` guard and the re-check, so the requirement applies to the combined case, or to a test that exposes the lock's effect.
- **R2d.** The assertion at `tests/api/newsletter.rs:2096-2101` must be able to fail. Its third disjunct, `!save_html.contains("Draft saved.")`, repeats the check at `:2092-2095`, so the rejection flash is never required.
- **R2e.** The Draft arm at `tests/api/newsletter.rs:2062-2064` accepts an outcome that correct code cannot produce: a seeded draft still in Draft after a 303 from publish-by-id. The test must not accept that outcome.

### R3. Flash sink must be escaped and covered (Finding 2d, PARTIAL)

- The suite must fail when escaping is removed from the flash path, that is, `flash_html` in `src/routes/admin/newsletter/get.rs` (currently `escape_attr` at `get.rs:24`).
- Current state: `&` and `'` are now asserted in the title attribute, both textareas, and both lists. The flash sink is not covered:
  - the flash text is fixed, server-authored, and contains no special characters, so no input reaches the sink;
  - the integration check at `tests/api/newsletter.rs:1856-1867` asserts only that no raw `<script>` appears, and its `<p><i>` presence check is always true because of the static paragraph at `get.rs:353`;
  - the unit tests at `get.rs:406-419` call `escape_attr`, not `flash_html`.
- Replacing `escape_attr(m.content())` with `m.content()` at `get.rs:24` currently passes the whole suite.

### R4. Malformed id in POST bodies and paths must be tested (Finding 2g-iii, PARTIAL)

- `POST /admin/newsletters` with a malformed `newsletter_issue_id` must have a test. The expected response is the one the handler specifies (400, returned before the idempotency check at `post.rs:86`). The suite must fail if that body is accepted.
- `POST /admin/newsletters/{newsletter_issue_id}/publish` with a non-UUID segment must have a test. It must not return 500 and must not enqueue anything.
- Current state: the draft-save POST with a malformed id is tested (`newsletter.rs:1926-1940`). Neither of the two cases above is.

## Not resolved

### R5. Malformed-path test must distinguish handled from unrouted (Finding 2g-ii, NOT RESOLVED)

- The malformed-path test at `tests/api/newsletter.rs:1923` must fail if the `/newsletters/{newsletter_issue_id}/edit` route is removed. Today it accepts the 404 that an unrouted path also returns, so it cannot tell the two apart.
- The comment at `tests/api/newsletter.rs:1921-1922` must describe actual behavior. It currently says a non-UUID path does not match the route. It does match, and the `Path<Uuid>` extractor rejects it with a 404.

## Residual

### R6. A stale Save draft must not silently replace a newer draft (Finding 1 residual, RESIDUAL)

- The commits fixed Publish: publish-by-id queues only stored content under `FOR UPDATE`, and compose Publish is insert-only.
- Save draft is unchanged. `save_newsletter_issue` (`post.rs:435-458`) upserts with no version or `updated_at` check.
- Requirement: a Save draft from a tab that loaded an older version must not overwrite a newer stored draft silently. The overwrite must be refused, or must be made visible to the admin.
- Requirement: a publish-by-id must not queue content that replaced a newer save without the admin being told.
- Scenario the suite must cover: Tab A loads v1, Tab B saves v2, Tab A saves v1, Tab A publishes. The stated requirement is what happens at the Tab A save.
- CLAUDE.md at lines 126-128 says "so a stale tab cannot send old form content". That is true only for the Publish button. Its wording must match whichever behavior is implemented.

### R7. Comments must match the race they describe (new, LOW)

- `src/routes/admin/newsletter/post.rs:346-350` describes the no-row race as a concurrent publish-by-id inserting and queueing. Publish-by-id never inserts; it returns 404 on a missing row (`post.rs:195-205`). The only path that reaches the no-row case is compose Publish on a compose-page id. The comment must name that path.

### R8. The tokio `sync` feature must be declared where it is used (new, LOW)

- Commit `7265038` added the `sync` feature to tokio under `[dependencies]` (`Cargo.toml:34`).
- Its only use is `tokio::sync::Barrier` in the test at `tests/api/newsletter.rs:1983`.
- Requirement: the feature's declaration scope must match its use. `Cargo.lock` is unchanged by this commit.

### R9. A key reused across two issue ids must not replay the other id's response (pre-existing, LOW)

- Idempotency keys are unique per user, with no issue id in the key (`migrations/20231110061915_create_idempotency_table.sql:14`).
- Reusing one key on two different issue ids returns the first request's saved redirect and flash to the second. The second issue is not queued.
- Not reachable from the UI today, because each form mint its own key. The requirement is that this cannot happen, whether or not the UI can reach it. A test must cover it.

## Evidence still required

These are not requirements on the code. They are gaps in what was checked.

- **Mutation evidence.** Each sensitivity requirement above (R1, R2, R3, R4, R5) must be confirmed by running the suite against the mutation it names and seeing the test fail. None has been run.
- **Race interleavings.** The race behavior in R2 has not been reproduced against a live database.
- **`cargo sqlx prepare --check`.** Not run; it needs a live database.
- **Browser behavior for Finding 3.** The leading-newline behavior was derived from the HTML parsing rules, not tested in a real browser.
- **Local `cargo test`.** Not run locally; the 22 lib and 83 api results come from CI on `7265038`.
