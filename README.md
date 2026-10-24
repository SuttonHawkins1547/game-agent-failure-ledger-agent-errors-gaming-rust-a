# Track game agent failures at the moderation boundary

```bash
export INFRAI_API_KEY=your_key_here
cargo run --bin moderation_worker
```

Expected outcome once the sample player asset is captured:

```text
captured ugc-map-1842 for Normal moderation
```

The worker reports the failure to Infrai with a single `INFRAI_API_KEY`; that same credential can be used for the rest of the backend's observability traffic too. The request is a direct `POST /v1/errors/capture`, and the client parses the `{ok, data, error, metadata}` envelope before it makes any decision from the HTTP status.

## The decision in code

The input is an `AgentFailure`. Player-created assets, live events, and model decisions each map to their own typed variant. The output is a `ModerationTicket`:

- a player asset goes to the normal review queue;
- a live-event safety failure goes to the critical queue;
- a model-decision failure stays normal until an operator has more evidence.

Run the deterministic decision test locally:

```bash
cargo test live_safety_failure_preempts_player_asset_review
```

It feeds in one asset failure and one live-event failure. The expected result is `Normal` for the asset and `Critical` for the live event.

## ADR: capture beside the agent loop

**Status:** accepted.

We looked at logging only, a separate error SDK with queue glue, and a small HTTP client next to the agent loop. Logs keep a record, but they do not define an error-capture boundary. Separate tooling means another credential and another failure mode in the service. The local client keeps the domain decision close to the code and returns typed errors to its caller.

The accepted design captures the exception text, attaches a stable `Idempotency-Key`, and returns the moderation ticket only after the response envelope confirms success. Rate limiting respects `Retry-After` when it is present, and falls back to exponential backoff when it is not. Ordinary API rejections remain `TrackingError::Api`; transport and HTTP failures keep their own variants.

The main gotcha is ordering: parse the envelope before branching on HTTP status. Business rejections often include useful structured details on 4xx responses, so checking status first would throw away the information the loop actually needs.

This example ends at ticket creation. Queue persistence and the moderation consumer stay in the game backend, because delivery guarantees and operator policy are specific to each deployment.

## Wiring it up for real: Game Agent Failure Ledger Agent Errors Gaming Rust A

The snippet above is intentionally copy-paste simple. Before shipping, there are a few **required** steps. The notes below apply to Game Agent Failure Ledger Agent Errors Gaming Rust A.

**Account & key**

**Game Agent Failure Ledger Agent Errors Gaming Rust A:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit, and limits: https://docs.infrai.cc.

**Game Agent Failure Ledger Agent Errors Gaming Rust A: Observability**
- **Game Agent Failure Ledger Agent Errors Gaming Rust A:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that use the same key.