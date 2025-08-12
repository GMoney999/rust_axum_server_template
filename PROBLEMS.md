# PROBLEMS

This document lists common problems you might encounter when developing or running this REST template server, along with simple, plain-language steps to fix them. Work through the items in order; most issues are resolved by following these checklists.

- Server won't start (crashes or exits immediately)
  1) Make sure dependencies are installed (e.g., run your package manager install command).
  2) Copy or set required environment variables (see .env.example if present).
  3) Check the startup command matches the project (e.g., run script in package.json or makefile).
  4) Read the first error message in the logs; fix the first error before chasing follow-on errors.
  5) Delete build artifacts and retry (e.g., clean the build/dist directory and rebuild).

- Port already in use (EADDRINUSE)
  1) Find what is using the port (e.g., lsof -i :PORT or netstat equivalent) and stop it.
  2) Or change this server's port using an environment variable (e.g., PORT=xxxx) or config file.
  3) Restart the server and verify it binds to the new/free port.

- Missing or invalid environment variables
  1) Check .env.example or documentation for required variables.
  2) Create a local .env file and fill in values; avoid quotes unless specifically required.
  3) Restart the server so changes take effect.
  4) If running via Docker, pass env vars through your compose or run command.

- Cannot connect to the database
  1) Verify the database is running and reachable from your machine/container.
  2) Check connection string/credentials in your env vars; confirm host, port, user, password, database name.
  3) Test connectivity using a CLI client with the same credentials.
  4) If using TLS, ensure certificates/params match the DB configuration.
  5) If using Docker, confirm both services are on the same network and the host is the service name.

- Database schema or migrations are out of date
  1) Run the migration command for this project to sync the schema.
  2) If migrations fail, read the first failing statement and fix data or constraints.
  3) If needed for local only, reset the local DB (drop/recreate) and re-run migrations.

- Requests fail with 4xx/5xx locally
  1) Check the server logs for the request; read the first error/stack trace.
  2) Confirm the route path and HTTP method are correct.
  3) Validate request body and headers match what the endpoint expects (e.g., Content-Type: application/json).
  4) Temporarily enable more verbose logging to see input and validation errors.

- CORS errors in the browser
  1) Confirm the frontend origin (protocol, host, port) is allowed in the server's CORS settings.
  2) Ensure preflight (OPTIONS) requests are handled and return the expected headers.
  3) For local dev, allow localhost origins or use a proxy that forwards requests.

- Health check endpoint reports unhealthy
  1) Check dependent services (database, cache, external APIs) are reachable.
  2) Verify any readiness checks (e.g., migrations applied) have passed.
  3) Restart the service after fixing dependencies.

- Hot reload not working
  1) Make sure you are running the dev/watch command, not the production build.
  2) Ensure the file watcher includes your source directories and excludes build output.
  3) On Docker/VMs, enable proper file sync or polling for file changes.

- Logs are missing or too noisy
  1) Set the log level via an environment variable or config (e.g., DEBUG, INFO, WARN, ERROR).
  2) For missing logs, ensure stdout/stderr are not redirected and the logger is initialized.
  3) For excessive logs, lower the level and disable verbose debug flags.

- TLS/HTTPS issues in development
  1) For local dev, prefer HTTP unless HTTPS is strictly required.
  2) If HTTPS is required, use dev certificates and trust them locally.
  3) Check that your proxy/load balancer forwards X-Forwarded-Proto if your app needs it.

- Docker build or run fails
  1) Confirm the base image and platform match your environment.
  2) Clear the Docker build cache and rebuild if dependencies changed.
  3) Ensure required build-time and run-time env vars are provided.
  4) If compose fails, bring all services down, remove volumes if needed, and start clean.

- Tests fail locally but pass elsewhere (or vice versa)
  1) Ensure you ran the same test command and environment as CI.
  2) Reset local state (databases, caches, temp files) before running tests.
  3) Run a single failing test in verbose mode and read the first failure.

- Linting or formatting blocks commits (pre-commit hooks)
  1) Run the linter/formatter locally and fix issues.
  2) If rules differ from your editor, configure your editor to use the project settings.
  3) Re-run the commit after fixes; do not skip hooks unless absolutely necessary.

- Rate limits or third-party API errors
  1) Check the API key and permissions are correct.
  2) Add exponential backoff and retry for transient errors in development.
  3) For testing, mock or stub external services when possible.

- Timeouts or slow responses
  1) Profile the endpoint to find slow operations (DB queries, external calls, large payloads).
  2) Add indexes to slow DB queries and avoid N+1 queries.
  3) Increase client/server timeouts only after fixing the root cause.

- Data not saving or being read back incorrectly
  1) Log the payload and validated data to confirm what is sent to the DB.
  2) Check transaction handling and error paths.
  3) Verify serialization/deserialization and field names match the schema.

- Configuration changes seem ignored
  1) Restart the server; some settings are only read at startup.
  2) Confirm you are editing the correct environment file/config for the current mode (dev/test/prod).
  3) Print the effective configuration at startup for debugging.

- Cross-environment differences (works on my machine)
  1) Pin exact dependency versions and lockfiles.
  2) Use the same runtime version (e.g., Node/Java/Python) across environments.
  3) Run within containers or dev environments that match CI and production.

- Authentication or session issues
  1) Confirm secrets/keys are set and consistent across app instances.
  2) Check token expiration, clock skew, and audience/issuer values.
  3) For cookies, verify domain/secure/samesite attributes in your environment.

- 404s for static files or API docs
  1) Ensure the static/docs directory is built and mounted correctly.
  2) Verify the static route or middleware is enabled in the server.
  3) Rebuild the docs/static assets if applicable.

If your issue isn’t covered here, capture the exact error message and the steps to reproduce, then open an issue or ask for help with those details.

