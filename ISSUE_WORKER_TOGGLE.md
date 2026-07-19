# Issue: Add config toggle to enable/disable the issue delivery worker

**Status:** Backlog  
**Priority:** Low  
**Related:** Database schema reorganization (PLAN.md)

## Description

Add a configuration flag to optionally disable the background email-delivery worker. This allows running the binary in API-only mode without processing the email delivery queue.

## Implementation

1. **Add config field** to `ApplicationSettings` in `src/configuration.rs`:
   - `worker_enabled: bool` — plain field, no `#[serde(default)]` (a bare `bool` default deserializes to `false`, i.e. worker-off, which is the wrong default). Follow the same pattern as the existing `tls_enabled: bool` field: no serde attribute, value always supplied by config.
   - Add `worker_enabled: true` to `configuration/base.yaml` under `application:` — this is what makes `true` the effective default.
   - Environment variable override: `APP_APPLICATION__WORKER_ENABLED` (nested under `application`, per the `APP` prefix / `__` separator convention — matches `APP_APPLICATION__PORT` etc.)

2. **Conditional spawn** in `src/main.rs`:
   - Skip spawning the worker task entirely when disabled
   - Only enter the `tokio::select!` loop if `worker_enabled` is true
   - Otherwise, await the application task directly
   ```rust
   if configuration.application.worker_enabled {
       let worker_task = tokio::spawn(run_worker_until_stopped(configuration.clone()));
       tokio::select! {
           o = application_task => report_exit("API", o),
           o = worker_task => report_exit("Background worker", o),
       };
   } else {
       application_task.await.ok();
   }
   ```

3. **Test:** Verify both `worker_enabled: true` and `worker_enabled: false` work without breaking the API server

## Future Enhancements

**Runtime toggle via admin endpoint:** Currently the toggle requires a container restart. A future enhancement could add an admin endpoint (e.g., `POST /admin/worker/disable`) that toggles worker state at runtime without restarting. This would require:
- An `AtomicBool` or DB flag for runtime state
- Worker loop checks the flag before processing tasks
- Admin endpoint to toggle it
- Note: This is out of scope for the initial implementation

## Notes

- Config changes require a container restart (no warm reload)
- This sets up flexibility for future operational scenarios (e.g., disabling worker during maintenance, or running separate containers for API vs. worker)
- No changes to the worker logic itself; the worker just won't be spawned if disabled
