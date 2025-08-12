# HTTP Routes

This document lists all HTTP endpoints exposed by the service, including methods, paths, headers, example curl requests/responses, and status codes.

Notes
- Base URL: depends on deployment (e.g., http://localhost:8000 or Shuttle URL)
- Authentication: If an ADMIN_TOKEN is configured at runtime, all routes require header Authorization: Bearer <ADMIN_TOKEN> and will return 401 Unauthorized when missing or invalid.
- Request ID: The server uses a request ID header (default x-request-id).
  - If you send this header, the same value is propagated.
  - If you omit it, the server generates one and returns it in the response.
- CORS: Configurable. Defaults are permissive for local/dev. This affects browser clients and preflight behavior, not the examples below.
- Trailing slashes are normalized; /todos and /todos/ are treated the same.


Endpoints

1) Health Check
- Method: GET
- Path: /health
- Required headers:
  - Authorization: Bearer <ADMIN_TOKEN> (only when auth is enabled)
- Optional headers:
  - x-request-id: <uuid>
- Example request:
  curl -i \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "x-request-id: 11111111-1111-1111-1111-111111111111" \
    http://localhost:8000/health
- Example response (200):
  HTTP/1.1 200 OK
  x-request-id: 11111111-1111-1111-1111-111111111111
  content-length: 22
  content-type: text/plain; charset=utf-8

  This is a health check
- Status codes:
  - 200 OK on success
  - 401 Unauthorized when Authorization is required and missing/invalid

2) List Todos
- Method: GET
- Path: /todos
- Required headers:
  - Authorization: Bearer <ADMIN_TOKEN> (only when auth is enabled)
- Optional headers:
  - x-request-id: <uuid>
- Example request:
  curl -i \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "x-request-id: 22222222-2222-2222-2222-222222222222" \
    http://localhost:8000/todos
- Example response (200):
  HTTP/1.1 200 OK
  content-type: application/json
  x-request-id: 22222222-2222-2222-2222-222222222222

  [
    {
      "id": 1,
      "title": "First task",
      "description": "Something to do",
      "done": false
    },
    {
      "id": 2,
      "title": "Second task",
      "description": "Another thing to do",
      "done": true
    }
  ]
- Status codes:
  - 200 OK on success
  - 401 Unauthorized when Authorization is required and missing/invalid
  - 500 Internal Server Error on database failures

3) Create Todo
- Method: POST
- Path: /todos
- Required headers:
  - Content-Type: application/json
  - Authorization: Bearer <ADMIN_TOKEN> (only when auth is enabled)
- Optional headers:
  - x-request-id: <uuid>
- Request body (application/json):
  {
    "title": "<string>",
    "description": "<string>",
    "done": <bool, optional, default false>
  }
- Example request:
  curl -i \
    -X POST \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -H "x-request-id: 33333333-3333-3333-3333-333333333333" \
    -d '{
      "title": "Buy milk",
      "description": "2% organic",
      "done": false
    }' \
    http://localhost:8000/todos
- Example response (200):
  HTTP/1.1 200 OK
  content-type: text/plain; charset=utf-8
  x-request-id: 33333333-3333-3333-3333-333333333333

  Successfully created user 123456789
- Status codes:
  - 200 OK on success
  - 400 Bad Request on invalid/undecodable JSON
  - 401 Unauthorized when Authorization is required and missing/invalid
  - 500 Internal Server Error on database failures


Models
- Todo (response):
  {
    "id": <number>,
    "title": <string>,
    "description": <string>,
    "done": <bool>
  }

- CreateTodo (request for POST /todos):
  {
    "title": <string>,
    "description": <string>,
    "done": <bool, optional>
  }


Environment and headers
- ADMIN_TOKEN: When present, all routes require Authorization: Bearer <ADMIN_TOKEN>.
- REQUEST_ID_HEADER: Name of the request ID header (default x-request-id). If changed, use that name in requests and expect it in responses.
- TIMEOUT_SECS: Global handler timeout (default 15s). Long requests may be terminated with a timeout by the server.
- CORS configuration affects browser calls (preflight); server defaults allow common headers (Content-Type, Authorization) and methods (GET, POST, PUT, PATCH, DELETE).

