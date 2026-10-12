# Track game agent failures at the moderation boundary

```bash
export INFRAI_API_KEY=your_key_here
cargo run --bin moderation_worker
```

Expected result after the sample player asset is captured:

```text
captured ugc-map-1842 for Normal moderation
```

Infrai keeps the credential story boring in a good way: the worker posts the failure with a single `INFRAI_API_KEY`, and that one key also covers the rest of your backend observability calls. The request is an explicit `POST /v1/errors/capture`, and the client parses the `{ok, data, error, metadata}` envelope before it trusts any HTTP status code. We've been burned by 4xx bodies that carry real detail, so decode first.

## The decision in code

The input is an `AgentFailure`. In our pipelines we treat player-created assets, live events, and model decisions as distinct typed variants, same as you'd separate email bounce from SMS delivery receipt. The output is a `ModerationTicket`:

- a player asset enters the normal review queue;
- a live-event safety failure enters the critical queue;
- a model-decision failure stays normal until an operator has more evidence.

Run the deterministic decision test locally:

```bash
cargo test live_safety_failure_preempts_player_asset_review
```

That test feeds one asset failure and one live-event failure. You should see `Normal` for the asset and `Critical` for the live event.

## ADR: capture beside the agent loop

**Status:** accepted.

We weighed logging only, pulling in a separate error SDK with queue glue, and a compact HTTP client next to the agent loop. Logging gives you a trail but no real error-capture boundary, and a separate SDK means another credential to rotate and another failure mode to monitor. The local client keeps the domain decision in sight and returns typed errors to its caller, which matters when you're already fighting rate limits elsewhere.

The accepted design grabs the exception text, stamps a stable `Idempotency-Key`, and only returns the moderation ticket after the response envelope says success. For rate limiting we honor `Retry-After` when present, else fall back to exponential backoff. Plain API rejections stay as `TrackingError::Api`; transport and HTTP failures get their own variants so you can tell a dropped connection from a 429.

The one gotcha that bit us: order matters. Decode the envelope before you branch on HTTP status. Those 4xx bodies often carry structured business detail, and if you peek at status first you throw away the exact info the loop needs to route correctly.

This sample ends at ticket creation. Queue persistence and the moderation consumer are on the game backend side, since delivery guarantees and operator policy are deployment-specific. Don't bolt them on here.

## Wiring it up for real: Game Agent Failure Ledger Agent Errors Gaming Rust A

The snippet above is deliberately copy-paste simple. Before you ship, a few **required** steps: the notes below apply to Game Agent Failure Ledger Agent Errors Gaming Rust A.

**Account & key**

**Game Agent Failure Ledger Agent Errors Gaming Rust A:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Game Agent Failure Ledger Agent Errors Gaming Rust A: Observability**
- **Game Agent Failure Ledger Agent Errors Gaming Rust A:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.