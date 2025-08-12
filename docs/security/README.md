# Security Guide

This document summarizes how ADMIN_TOKEN bearer authentication is used in this service, the implications of CORS when exposing HTTP endpoints, and basic threat considerations with recommended mitigations.

## Overview

- The service protects administrative endpoints with a static bearer token provided via the Authorization header.
- Browsers enforce Cross-Origin Resource Sharing (CORS) rules; servers must respond with explicit headers to allow cross-origin requests.
- Treat the ADMIN_TOKEN like a password with full administrative scope. Leakage grants full control to an attacker.

## ADMIN_TOKEN bearer authentication

### How it works

- Admin-only endpoints require an Authorization header using the Bearer scheme:
  - Authorization: Bearer <ADMIN_TOKEN>
- If the header is missing or malformed, the service returns 401 Unauthorized.
- If the token value is present but incorrect, the service returns 403 Forbidden.
- The token is expected to be provided by trusted automation/CLI or a backend service. It must not be embedded in front-end code or exposed to browsers.

### Configuration

- The token is supplied via the ADMIN_TOKEN environment variable at process startup.
- On boot, the service reads the token into memory. If ADMIN_TOKEN is unset or empty, admin endpoints should be disabled or startup should fail fast.
- When rotating tokens, prefer a short overlap window and restart/reload the service so the new token is in effect.

### Usage examples

- Curl:
  - curl -H "Authorization: Bearer ${ADMIN_TOKEN}" https://api.example.com/admin/health
- Programmatic (pseudo-code):
  - headers["Authorization"] = "Bearer " + adminToken

### Operational guidance

- Storage: Keep ADMIN_TOKEN in a secrets manager or encrypted environment. Never commit it to VCS or store it in plaintext config.
- Rotation: Rotate on a regular cadence (e.g., 90 days) and immediately on suspected compromise.
- Scope: This implementation uses a single static token with full admin rights. Be aware this is high risk compared to per-user tokens with RBAC.
- Audit: Log admin access attempts, including endpoint, outcome (success/denied), and source IP. Avoid logging the token itself.
- Rate limiting: Apply conservative rate limits to admin endpoints to reduce brute-force risk.
- Process environment hygiene: Ensure tooling and error reporters do not dump environment variables to logs.

### Local development

- Use a distinct, non-production token for local/dev environments.
- Prefer .env files that are excluded from version control (e.g., via .gitignore). Example:
  - ADMIN_TOKEN=dev-only-change-me

## CORS implications

CORS is a browser security mechanism governing whether a web page from one origin can make requests to another origin. Key points:

- CORS is enforced by browsers, not by servers. A permissive server CORS policy does not reduce risk for non-browser clients; curl and serverside code can make requests regardless of CORS.
- CORS does not provide authentication or authorization. Do not rely on CORS to protect admin endpoints.
- If a browser front-end must call this API cross-origin and include the Authorization: Bearer header, the server must:
  - Echo Access-Control-Allow-Origin with the specific allowed origin (avoid using "*")
  - Include Access-Control-Allow-Headers with Authorization
  - Respond to preflight OPTIONS requests with the above headers
  - If credentials (cookies, Authorization header) are used, do not use wildcard origins and set Access-Control-Allow-Credentials: true

### Recommended CORS settings

- Restrict allowed origins to a specific list of trusted domains (e.g., https://app.example.com).
- Allow only necessary methods (e.g., GET, POST, PATCH, DELETE) and headers (Authorization, Content-Type).
- Disable or narrowly scope CORS for admin endpoints entirely when possible; admin calls should be made from server-to-server contexts, not browsers.
- Never combine Access-Control-Allow-Origin: * with Access-Control-Allow-Credentials: true (disallowed by browsers and unsafe by design).

### Example preflight response headers

- Access-Control-Allow-Origin: https://app.example.com
- Access-Control-Allow-Methods: GET, POST, PATCH, DELETE, OPTIONS
- Access-Control-Allow-Headers: Authorization, Content-Type
- Access-Control-Allow-Credentials: true
- Access-Control-Max-Age: 600

## Basic threat notes and mitigations

### Token theft/leakage

- Vectors: logs, source control, browser storage (localStorage/sessionStorage), referrers, error reporting, chat pastes, screenshots.
- Mitigations: never log tokens, scrub headers in logs and traces, store only in server memory or a secure secret manager, use short rotation intervals.

### Brute force/credential stuffing

- Vectors: automated guessing of ADMIN_TOKEN via public endpoints.
- Mitigations: rate limit and IP throttle admin routes; implement exponential backoff on failures; alert on repeated 401/403 events.

### Transport security

- Always serve admin endpoints over HTTPS; enforce HSTS at the domain level.
- Reject cleartext HTTP or downgrade attempts.

### Replay attacks

- A static bearer token is replayable by design. Prefer to keep admin endpoints out of the browser and restrict by network perimeter or IP allowlist.
- Consider moving to expiring tokens (e.g., signed JWT with short TTL) or mTLS for high-assurance environments.

### CSRF/XSS considerations

- Bearer tokens are not automatically attached by browsers like cookies, so CSRF risk is lower for pure Authorization headers.
- If the same origin also uses cookies for any admin session, implement CSRF protections (SameSite=strict/lax, CSRF tokens).
- Prevent XSS in any admin UI; XSS can exfiltrate tokens or perform actions as the admin user.

### Denial of Service

- Protect expensive admin operations with authentication checks early in request handling.
- Apply request size limits and timeouts; use circuit breakers to downstream dependencies.

### Supply chain and dependency risk

- Pin dependency versions, enable vulnerability scanning, and patch promptly.
- Treat build and CI secrets with the same care as ADMIN_TOKEN.

### Observability and alerting

- Emit security-relevant events: auth failures, token rotation, configuration changes.
- Add alerts on spikes in 401/403, anomalous IP ranges, or unexpected admin endpoint usage times.

## Checklist

- [ ] ADMIN_TOKEN provided via environment variable in all deployments
- [ ] Admin endpoints reject missing/invalid tokens with 401/403 and do not leak details
- [ ] No tokens in logs, analytics, or error reports
- [ ] CORS restricted to trusted origins; Authorization header allowed only where needed
- [ ] Admin endpoints not exposed to browsers (or CORS disabled for them)
- [ ] HTTPS enforced; HSTS enabled
- [ ] Rate limiting and monitoring on admin routes
- [ ] Documented and tested token rotation procedure

## Future improvements

- Replace static ADMIN_TOKEN with per-user identities and short-lived tokens (OIDC/JWT) and RBAC.
- Add mTLS and/or IP allowlisting for admin routes.
- Introduce audit logging with tamper-evident storage.
