# Observability

This document describes how tracing is set up, how request IDs are handled for cross-system correlation, and how to enable/export OpenTelemetry data.

## Tracing setup

Our tracing is based on OpenTelemetry (OTel) and is designed to work in any environment (local, CI, staging, production).

- Initialization
  - A tracer provider is initialized at application startup.
  - The service name, environment, and version are attached as resource attributes to all spans.
  - A parent context is extracted from inbound requests using W3C Trace Context (traceparent/tracestate) and Baggage when present.
  - If no parent is present, a new trace is started.

- Instrumentation
  - Incoming HTTP/gRPC requests are wrapped in middleware/interceptors that:
    - Extract the trace context and start a server span (kind=server) per request.
    - Attach request-scoped attributes (method, route, status code, user agent, remote IP when appropriate, request ID).
  - Outbound calls (HTTP, gRPC, DB, message bus) are wrapped by client instrumentation that:
    - Starts a child span (kind=client or kind=producer).
    - Injects the current trace context on the wire so downstream services can join the same trace.
  - Errors are recorded on spans with appropriate status and exception attributes.

- Span conventions
  - Name server spans as: HTTP <METHOD> <ROUTE> (e.g., HTTP GET /api/orders/{id}).
  - Name client spans as: <OPERATION> <DESTINATION> (e.g., HTTP GET inventory-service /items/{id}).
  - Attach semantic attributes per OpenTelemetry conventions (http.*, db.*, messaging.*, net.*, enduser.* where applicable).

- Sampling
  - Default sampler is parent-based with a configurable root sampler (typically traceidratio-based).
  - Sampling rate can be adjusted via environment variable without redeploy.
  - For local development, you can set 100% sampling to make debugging easier.

- Context propagation
  - We use W3C Trace Context headers by default: `traceparent` and `tracestate`.
  - Baggage is supported for lightweight key/value metadata that needs to flow between services. Avoid putting PII or large payloads in baggage.

## Request IDs

Request IDs are used for log correlation and to map application logs to traces.

- Generation
  - On inbound requests, the system looks for an existing request ID header (see Propagation) and uses it when present.
  - If absent, a new cryptographically strong request ID is generated at the edge (ingress/middleware).

- Propagation
  - The request ID is carried in the `X-Request-ID` header (primary) and mirrored to logs.
  - For gRPC or other protocols, an equivalent metadata key (e.g., `x-request-id`) is used.
  - Outbound requests automatically forward the current request ID header to downstream services.

- Correlation with traces
  - The request ID is added as a span attribute (e.g., `http.request_id`) on the top-level server span and to child spans when practical.
  - Logs include the request ID and, when available, the trace ID and span ID, enabling end-to-end correlation across logs and traces.

- Logging format
  - Structured logs should include fields: `request_id`, `trace_id`, `span_id`, `severity`, `message`, and key request attributes (method, path, status).

## OpenTelemetry exporter note

By default, the application uses OpenTelemetry SDKs and will operate even without an external backend. Export behavior is controlled by environment variables so you can enable it without code changes.

- Default behavior
  - If no exporter is configured, spans are created in-process and dropped (no-op exporter). This keeps overhead minimal.

- Enabling export (recommended)
  - Set OTLP endpoint and protocol to export spans/metrics/logs to a collector or backend (Tempo, Jaeger via collector, Honeycomb, New Relic, etc.). Example env vars:
    - OTEL_SERVICE_NAME: service identifier (e.g., orders-api)
    - OTEL_EXPORTER_OTLP_ENDPOINT: e.g., http://localhost:4318 or http://otel-collector:4317
    - OTEL_EXPORTER_OTLP_PROTOCOL: http/protobuf (4318) or grpc (4317)
    - OTEL_RESOURCE_ATTRIBUTES: service.version=1.2.3,deployment.environment=staging
    - OTEL_TRACES_SAMPLER: parentbased_traceidratio
    - OTEL_TRACES_SAMPLER_ARG: 1.0 (for 100% in dev) or 0.1 (10% in prod)
    - OTEL_LOGS_EXPORTER/OTEL_METRICS_EXPORTER: otlp or none
  - If your backend requires authentication, configure headers via:
    - OTEL_EXPORTER_OTLP_HEADERS: Authorization=Bearer {{OTEL_API_TOKEN}}

- Local development quickstart
  1) Run a local collector or backend (e.g., `docker run -p 4317:4317 -p 4318:4318 otel/opentelemetry-collector:latest` with a basic config).
  2) Export to the collector by setting:
     - OTEL_SERVICE_NAME=dev-app
     - OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
     - OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
     - OTEL_TRACES_SAMPLER=parentbased_traceidratio
     - OTEL_TRACES_SAMPLER_ARG=1.0
  3) Trigger requests and verify traces in your chosen backend (Tempo/Grafana, Jaeger via collector, etc.).

- Performance and privacy
  - Tracing adds small overhead. Use sampling in production and avoid attaching large attributes.
  - Do not put secrets, tokens, or PII in span names, attributes, logs, or baggage.

## Operational guidance

- Rollouts
  - Treat changes to sampling and exporter endpoints as config changes; roll them out gradually.

- Troubleshooting
  - No traces in backend: verify OTEL_EXPORTER_OTLP_ENDPOINT and protocol, ensure network egress, check collector logs.
  - Context not propagating: confirm inbound middleware and outbound interceptors are applied; ensure proxies/gateways forward headers.
  - Request ID missing: verify middleware is mounted early in the pipeline and that downstream services are not overriding the header.

## Summary

- Tracing is enabled via OpenTelemetry with W3C propagation and semantic conventions.
- Request IDs are generated/propagated via X-Request-ID and logged for correlation.
- Exporting is disabled by default; enable OTLP export via environment variables to send data to your preferred backend.
