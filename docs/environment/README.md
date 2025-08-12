# Environment and Secrets

This document explains how to manage configuration for this service in two contexts:
- Local development with regular environment variables (e.g., a .env file or shell exports)
- Deployment on Shuttle using its Secret Store

It also includes practical ADMIN_TOKEN examples for both environments.


## TL;DR
- Local: put non-sensitive config into a local .env and export in your shell, but avoid storing real secrets there. Use placeholders for examples.
- Shuttle: store real secrets (like ADMIN_TOKEN) with Shuttle’s Secret Store. Do not hardcode them in code or commit them to the repo.
- If ADMIN_TOKEN is present at runtime, all routes are protected by a static Bearer token. Clients must send: Authorization: Bearer <token>.


## What goes where?
- Regular env vars (not secret):
  - REQUEST_ID_HEADER, TIMEOUT_SECS, CORS_DISABLED, CORS_ALLOWED_ORIGINS, RUST_LOG
  - Recommendation: manage locally via .env; in Shuttle deployments, set them using Shuttle environment configuration or as part of your deployment workflow.
- Secrets:
  - ADMIN_TOKEN (protects all routes if present)
  - Store only in Shuttle Secret Store for deployed environments. For local testing, either set a temporary shell variable or use a local secret manager.


## Local development
You can run the server locally using environment variables. Two convenient approaches:

1) Temporary per-command exports
- Unix shells
  - Example: TIMEOUT_SECS=15 RUST_LOG=info cargo run
- PowerShell
  - Example:
    - $env:TIMEOUT_SECS = "15"
    - $env:RUST_LOG = "info"
    - cargo run

2) .env file (not committed)
- Create a .env file at the project root (and ensure .gitignore excludes it).
- Example .env for local use:

REQUEST_ID_HEADER=x-request-id
TIMEOUT_SECS=15
# Leave CORS permissive by default in dev
# CORS_DISABLED=
# CORS_ALLOWED_ORIGINS=
RUST_LOG=info,tower_http=info,sqlx=warn

# Optional: local-only admin token for manual testing (do NOT commit real values)
# ADMIN_TOKEN=dev-local-admin-token

- Load this into your shell session before running cargo (one-time per session):
  - Unix shells: set -a; source .env; set +a
  - PowerShell: Get-Content .env | ForEach-Object { if ($_ -match '^(.*?)=(.*)$') { $name=$matches[1]; $value=$matches[2]; [Environment]::SetEnvironmentVariable($name,$value,'Process') } }

Note: If you include ADMIN_TOKEN locally, every request to your server must include the Authorization header. Remove it from your environment to disable auth locally.


## Using ADMIN_TOKEN locally
- Start the server with ADMIN_TOKEN set (see above).
- Call an endpoint with the header:

curl -H "Authorization: Bearer {{ADMIN_TOKEN}}" http://localhost:8000/health

Replace {{ADMIN_TOKEN}} with your local token value set in the environment (do not paste secrets into your shell history; prefer using variables).


## Shuttle secrets (deployment)
Shuttle provides a managed Secret Store. Use it for all sensitive values in deployed environments.

Common operations:
- Add or update a secret:
  shuttle secrets add ADMIN_TOKEN
  # You will be prompted to input the value securely

- List secret names (not values):
  shuttle secrets list

- Remove a secret:
  shuttle secrets delete ADMIN_TOKEN

- Clear all secrets (use with caution):
  shuttle secrets clear

During deployment and runtime on Shuttle, the service will read ADMIN_TOKEN from the Secret Store. You should not define ADMIN_TOKEN as a plain environment variable in production.


## Using ADMIN_TOKEN on a deployed service
- Ensure you have set the secret on Shuttle as described above.
- When making requests to the deployed URL, include the header:

Authorization: Bearer {{ADMIN_TOKEN}}

Example curl (replace with your deployment’s base URL):

curl -H "Authorization: Bearer {{ADMIN_TOKEN}}" https://<your-shuttle-app>.shuttleapp.rs/health

Never print the token to logs or share it in plaintext. Use a variable in your CI/CD systems and developer tooling.


## Rotation and hygiene
- Rotate ADMIN_TOKEN periodically or when team membership changes.
- Prefer strong, random values. A 32+ byte random token (base64/hex) is a good baseline.
- Keep secrets out of commit history. Provide a .env.example that shows variable names with placeholder values, not real ones.
- Scope access: only provide tokens to roles that need them.


## Troubleshooting
- 401 Unauthorized locally
  - Ensure ADMIN_TOKEN is not set (if you expect open access) or that you are sending the correct Authorization header.
- 401 Unauthorized on Shuttle
  - Verify the secret exists: shuttle secrets list
  - Confirm your client is using the correct token.
- CORS behavior seems wrong
  - Check CORS_DISABLED and/or CORS_ALLOWED_ORIGINS values in your environment.


## Reference: variables recognized by the service
- REQUEST_ID_HEADER: header used for request correlation (default: x-request-id)
- TIMEOUT_SECS: per-request timeout in seconds (default: 15)
- CORS_DISABLED: if set, disables CORS entirely
- CORS_ALLOWED_ORIGINS: comma-separated allow-list of origins
- RUST_LOG: tracing filter, e.g., info,tower_http=info,sqlx=warn
- ADMIN_TOKEN (secret): enables global bearer token auth when present
