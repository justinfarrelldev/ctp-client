# Server Startup Investigation Plan

Branch: `singleplayer-in-game`

## Checklist

- [x] Confirm the failing address (`127.0.0.1:54322`) owner from server configuration/docs.
  - It is the local Supabase Postgres port from `$CTPSERVER/supabase/config.toml` and `.env` `SUPABASE_DB_URL`.
- [x] Inspect Open CTP server startup tasks and environment defaults.
  - `task run` starts `air`; `main.go` loads `.env`, calls `sqlx.Open`, then background gameplay workers query Postgres.
- [x] Determine whether the dependency is local Supabase/Postgres and how it is meant to start.
  - README requires `supabase start` before `task run`, then `.env` values from Supabase CLI.
- [x] Check whether a local process is listening on the expected port.
  - At investigation time port `54322` was occupied by a different Supabase project (`lifted-resume`), while `supabase status` in `$CTPSERVER` expected `supabase_db_open-ctp-server` and failed.
- [x] Provide exact remediation commands and any follow-up code/doc changes if needed.
  - Stop the wrong Supabase project or all local Supabase stacks, then start Supabase from `$CTPSERVER`, reset migrations if needed, then start `task run`.
- [x] Run relevant verification commands if code/docs are changed.
  - No source code changes were needed for the diagnosis.
