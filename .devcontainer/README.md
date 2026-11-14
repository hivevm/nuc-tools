# Development observability stack

OTLP in, Grafana out: Claude Code's own metrics and events, stored in ClickHouse. It is a Compose
project of its own ([`compose.yaml`](compose.yaml)) that runs on the host, apart from the Dev
Container: start it when you want to look, leave it off when you do not. This is a **development
tool**: nothing shipped depends on it. It holds personal data, though: every row carries the
account's email address ([Pointing Claude Code at it](#pointing-claude-code-at-it)). It keeps its
data on a local volume: metrics for 90 days, logs and traces for 24 h ([Retention](#retention)).

```
Claude Code ──OTLP/HTTP:4318──▶ OpenTelemetry Collector ──▶ ClickHouse ──▶ Grafana :3001
                                            otel_metrics_sum · otel_logs · otel_traces
```

## Why one store

All three signals sit in one SQL database instead of one backend per signal (Prometheus, Loki,
Tempo):

- **One query language.** Every panel is SQL.
- **Cross-signal questions become joins** — `otel_logs` and `otel_traces` share `TraceId`.
- **One place for retention.** The `ttl` of each exporter in
  [otel-collector.yaml](otel-collector.yaml).

What it costs:

- **The datasource is a plugin.** `grafana-clickhouse-datasource` is not built into Grafana, so the
  first `up` needs network access to install it, and it requires Grafana ≥ 11.6.0.
- **No PromQL.** A dashboard copied from elsewhere has to be rewritten.
- **The Collector is not optional.** ClickHouse speaks no OTLP; if the Collector is down, nothing
  is stored.

## Start

On the **host**, from the repository root (Podman's `podman compose` takes the same arguments):

```sh
docker compose -f .devcontainer/compose.yaml up -d      # start, in the background
docker compose -f .devcontainer/compose.yaml down       # stop; the data is kept
docker compose -f .devcontainer/compose.yaml down -v    # stop, and throw the data away
```

Or `cd .devcontainer` and leave out the `-f`. The stack needs roughly 2 GB of memory.

Grafana is on <http://localhost:3001> and opens on **Claude Code — Overview**. Anyone who reaches
it may look without a login; editing a dashboard takes the login `admin`/`admin`, which Grafana
asks to change on the first sign-in.

**The ports are published on the host's loopback only**, since Grafana and ClickHouse hold
personal data behind no credential worth the name. Docker Desktop forwards
`host.docker.internal` to the host's loopback, so the Dev Container reaches them as they are;
whether Podman does depends on its network mode. Docker Engine on Linux does not: there, bind them
to the bridge gateway instead, which only the host and its containers reach:

```sh
OBSERVABILITY_BIND=172.17.0.1 docker compose -f .devcontainer/compose.yaml up -d
```

Check with the `curl` under [Checking that data arrives](#checking-that-data-arrives) from inside
the container.

### Reaching it from the Dev Container

The Dev Container reaches the Collector **through the host**, over the ports published above; it
needs no Docker socket for that.

| From | Collector (OTLP/HTTP) | Grafana | ClickHouse |
| ---- | --------------------- | ------- | ---------- |
| the host | `http://127.0.0.1:4318` | <http://localhost:3001> | `localhost:8123` |
| the Dev Container, Podman or Docker Desktop | `http://host.docker.internal:4318` | the host's browser | `host.docker.internal:8123` |
| the Dev Container, Docker Engine on Linux | `http://172.17.0.1:4318` | the host's browser | `172.17.0.1:8123` |

Podman also answers to `host.containers.internal`. Docker Engine on Linux gives the host no name;
the container reaches it through the bridge gateway, which `ip route` inside the container shows as
the default route (`default via 172.17.0.1` on the default bridge). A host firewall that filters
the `docker0` interface blocks it there; allow the published ports from the bridge.
`--add-host=host.docker.internal:host-gateway` in the container's `runArgs` gives the gateway the
name the other engines use.

## Pointing Claude Code at it

Claude Code exports when its environment says so. Put the variables into
`~/.claude/settings.json`. Claude Code ignores these keys in `.claude/settings.json` and
`.claude/settings.local.json`, and logs a warning naming each one. The setting applies to every
project that uses this configuration directory:

```json
{
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",
    "OTEL_METRICS_EXPORTER": "otlp",
    "OTEL_LOGS_EXPORTER": "otlp",
    "OTEL_EXPORTER_OTLP_PROTOCOL": "http/protobuf",
    "OTEL_EXPORTER_OTLP_ENDPOINT": "http://host.docker.internal:4318",
    "OTEL_METRIC_EXPORT_INTERVAL": "10000",
    "OTEL_LOGS_EXPORT_INTERVAL": "5000",
    "OTEL_METRICS_INCLUDE_REPOSITORY": "1",
    "CLAUDE_CODE_ENHANCED_TELEMETRY_BETA": "1",
    "OTEL_TRACES_EXPORTER": "otlp"
  }
}
```

The endpoint is the one from the table above for your engine — `http://172.17.0.1:4318` on Docker
Engine on Linux; on the host it is `http://127.0.0.1:4318`. Start a new session after changing the
variables: only that is sure to read them all. A running session may still take up a change, as it
does `OTEL_METRICS_INCLUDE_REPOSITORY`, and then its earlier rows stay without the new value.

Things this stack is shaped around:

- **Delta temporality.** Claude Code's default for metrics. Every sum panel adds the exported
  deltas up; with `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE=cumulative` they would count
  the same usage again on every export.
- **No content by default.** Prompts and tool parameters are left out unless
  `OTEL_LOG_USER_PROMPTS=1` or `OTEL_LOG_TOOL_DETAILS=1` is set. The export is plaintext HTTP and
  ClickHouse keeps it for 24 h; turn those on deliberately.
- **Identity is always there.** Metrics and events carry `user.email`, `user.account_uuid`, and
  `organization.id` whatever the settings above say, so the store is personal data to anyone who
  reaches its ports.
- **Traces are beta.** The last two variables above send them to `otel_traces`. Each prompt is a
  tree of `claude_code.interaction`, `.llm_request`, `.tool`, `.tool.blocked_on_user`, and
  `.tool.execution` spans. Spans carry no cost: `request_id` and `tool_use_id` join them to the
  events. A `Bash` tool span names the program it ran in `bash_argv0`, without its arguments.

## The schema

The Collector's ClickHouse exporter owns it — `create_schema: true` runs `CREATE TABLE IF NOT
EXISTS` at startup, so no `.sql` file here has to be kept in step with the exporter's column list.

| Table | Holds | Time column |
| ----- | ----- | ----------- |
| `otel.otel_metrics_sum` | Claude Code's counters — `claude_code.cost.usage`, `.token.usage`, `.session.count`, `.active_time.total`, `.lines_of_code.count`, `.commit.count`, `.pull_request.count`, `.code_edit_tool.decision` | `TimeUnix` |
| `otel.otel_logs` | Claude Code's events, named in `LogAttributes['event.name']` — `api_request`, `api_error`, `tool_result`, `tool_decision`, `hook_execution_complete`, `user_prompt`, … | `Timestamp` |
| `otel.otel_traces` | Spans, when the beta is on; `Duration` in nanoseconds | `Timestamp` |

The exporter creates its other metric tables too (`otel_metrics_gauge`, `otel_metrics_histogram`,
…); Claude Code writes only counters, so they stay empty.

Metric names are stored unmodified, with no Prometheus-style normalisation. A metric's own
attributes — `model`, `type`, `session.id` — are in `Attributes`; an event's — `tool_name`,
`duration_ms`, `cost_usd`, `tool_result_size_bytes`, `hook_name`, `total_duration_ms`, all as
strings — in `LogAttributes`. The panels also accept an event name with the `claude_code.` prefix.
The `vcs.*` attributes are in those two as well, not in `ResourceAttributes`. `ServiceName` is
`claude-code`.

## Reading the data

### When a row appears

The two intervals say how often Claude Code sends, not how often something is measured:

- **Metrics.** Every `OTEL_METRIC_EXPORT_INTERVAL`, the sum since the last export goes out, and only
  if it changed. An idle session writes no rows, so the gaps in `TimeUnix` are as long as the idle
  time, on a grid of the interval.
- **Events.** An event is created when the thing happens. `OTEL_LOGS_EXPORT_INTERVAL` only says when
  it is sent; `Timestamp` stays the moment it happened.

The Collector then batches for up to 5 s, so with the settings above a metric lands in ClickHouse
within about 15 s and an event within about 10 s.

### Which project and which agent

| Attribute | On | Says |
| --------- | -- | ---- |
| `vcs.repository.name`, `vcs.owner.name`, `vcs.repository.url.full`, `vcs.provider.name` | metrics, events | The repository, with `OTEL_METRICS_INCLUDE_REPOSITORY=1`, derived from the `origin` remote |
| `query_source` | cost and token counters | `main`, `subagent`, or `auxiliary`; events carry finer values of their own |
| `agent.name` | cost and token counters, API events | The subagent type, absent when no named subagent made the request |
| `agent_id`, `parent_agent_id` | events only | Which subagent instance, and which agent started it; absent in the main session |
| `ServiceName` | every row | Which tool sent it, should anything but Claude Code export here |

The dashboard's **Project** filter and its **Cost by project and agent** panel read
`vcs.repository.name`:

- **`(none)`** stands for rows without it: a session with no `origin` remote, or rows from before
  the setting took effect. A session that took it up while running shows under both, so the
  **Sessions** column need not add up.
- **The name alone** is what is filtered on, so repositories of two owners with one name share a
  row; `vcs.owner.name` tells them apart in a query.
- **A directory with no `origin`** can name itself in `OTEL_RESOURCE_ATTRIBUTES`, as
  `project=name`. A space in a value makes Claude Code drop every custom key, without an error.

`agent.name` shows built-in agents and those of official-marketplace plugins by name, and any other
as `custom`. `OTEL_LOG_TOOL_DETAILS=1` gives the real names, and stores Bash commands and tool input
along with them.

### What a panel does not cover

**Hook duration** shows Claude Code's own hooks. The git hooks of a repository run inside a `Bash`
call and count towards that call's duration in **Tool calls**.

**Tool result size** is in bytes, not tokens, and leaves out calls that report no size.

**The panels per hour and per day** keep their own window of 48 h and 30 days, whatever the
dashboard's time range. Their days and hours follow the browser's time zone, and the oldest bar of
a panel per day covers only part of its day. Cost and tokens land in the hour they were exported.

## Retention

Metrics stay for 90 days, logs and traces for 24 h: the `ttl` of the two exporters in
[otel-collector.yaml](otel-collector.yaml). Metrics also carry the identity named above, but no
prompts or tool input. At the rate measured on 2026-11-08, about 48 KiB in two busy hours, 90 days
of steady use take some 50 MB.

The exporter sets a TTL only when it creates a table, so a store that already exists keeps its old
one. After a change, restart the Collector and set the TTL once by hand; data already deleted does
not come back:

```sh
docker compose restart otel-collector   # on the host, in this directory
curl -s 'http://localhost:8123/?user=otel&password=otel' --data-binary \
  "ALTER TABLE otel.otel_metrics_sum MODIFY TTL toDateTime(TimeUnix) + toIntervalDay(90)"
```

## What is already there

The dashboards in `grafana/dashboards/` are provisioned into the **ClaudeCode** folder:

| Dashboard | Shows |
| --------- | ----- |
| **Claude Code — Overview** | Cost, tokens, sessions, active time, lines added and commits for the window; cost by model and tokens by type over time; cost and tokens by project per hour and per day; the cache hit rate, the cost per session, the cost by project and agent, as a table and as shares in two pie charts, and the most expensive prompts; API request duration and errors; tool calls with failures and p95, who decided each permission, the size of tool results, and the duration of Claude Code's hooks; the slowest prompts, each opening its trace, and the time spent waiting for approval; a tail of the events. Filterable by project and session. |

Drop any further dashboard JSON into `grafana/dashboards/` — it is picked up within 30 s, no
restart. [`scripts/check-dashboards.sh`](../scripts/check-dashboards.sh) holds it to the rule the
overview follows: every panel query filters by `project` and `session`, on the whole name.

## Starting from an empty store

ClickHouse deduplicates nothing, so to start clean (its HTTP interface takes one statement per
request):

```sh
for table in otel_logs otel_traces otel_metrics_sum; do
  curl -s 'http://localhost:8123/?user=otel&password=otel' --data-binary "TRUNCATE TABLE otel.$table"
done
```

## Checking that data arrives

```sh
# Does the Collector accept OTLP at all? (from the Dev Container: host.docker.internal)
curl -i -X POST http://127.0.0.1:4318/v1/logs \
     -H 'Content-Type: application/json' --data '{"resourceLogs":[]}'

# What is the Collector itself doing?
docker compose -f .devcontainer/compose.yaml logs -f otel-collector   # on the host

# What made it into ClickHouse?
curl -s 'http://localhost:8123/?user=otel&password=otel' --data-binary "
  SELECT 'metrics' AS signal, count() AS rows, max(TimeUnix)  AS newest FROM otel.otel_metrics_sum
  UNION ALL
  SELECT 'logs',              count(),        max(Timestamp)         FROM otel.otel_logs
  UNION ALL
  SELECT 'traces',            count(),        max(Timestamp)         FROM otel.otel_traces"
```

If a dashboard stays empty but the counts above are non-zero, the fault is in the panel, not the
pipeline. If the counts are zero, the Collector is the place to look — add `debug` to the
`exporters` of the relevant pipeline in [otel-collector.yaml](otel-collector.yaml) and restart that
one service, and it prints what it receives.

## Ports

| Port | Service | What for |
| ---- | ------- | -------- |
| 4318 | Collector | OTLP/HTTP — what Claude Code exports to |
| 4317 | Collector | OTLP/gRPC — for anything else you want to point here |
| 3001 | Grafana | The UI |
| 8123 | ClickHouse | HTTP interface — `curl`-able, and what the checks above use |

The Collector's own metrics (8888), its health check (13133), and ClickHouse's native protocol
(9000) stay on the Compose network: the Collector and Grafana reach ClickHouse there, and nothing
on the host needs the others.
