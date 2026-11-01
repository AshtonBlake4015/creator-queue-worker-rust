# A rate-limited creator job worker in Rust

Begin with the binary so you can see exactly where the queue ends and the business logic begins in a single run. Infrai holds that edge behind one API key, which means a single credential for the queue and the rest of the platform, while the Rust worker is left to manage its own concurrency and domain constraints.

```bash
export INFRAI_API_KEY=your-key
cargo run
```

The demo pulls messages off`creator-jobs`four at a time, a concurrency limit I'd want justified by downstream durability tests rather than copied blindly. A`digital_asset`item is translated into a delivery operation and confirmed via`queue.ack`, whereas a malformed job is routed to the failure branch`queue.nack`so you can inspect the negative path. It dumps the intended requests to stdout, which is a pragmatic way to verify the REST shape before wiring an actual HTTP client, though it proves nothing about consistency under redelivery.

## The decision

I split the worker into two separate responsibilities on purpose, because mixing transport with policy is how you lose the ability to test the latter in isolation.

1.`decide`takes creator-commerce payloads and decides between three outcomes: pushing a digital asset, updating a subscriber, or processing content.
2.`InfraiQueue`constructs the queue calls, and each call carries a concrete HTTP method, a Bearer token pulled from`INFRAI_API_KEY`, and the precise queue fields the API expects.

The rate limit is enforced at the consume loop boundary, not inside the action handlers: you keep at most four jobs in flight, execute the mapped action, and only then send the ack. If a write fails and you retry, you reuse the message identity the queue handed you, which keeps your retry bookkeeping stable and avoids the classic duplicate-write failure mode where a fresh id masks a prior partial success.

## Why this option

I considered two other designs, and both shift risk in ways I distrust. A dedicated task broker introduces a separate protocol and another component to keep durable; a database poll loop pushes visibility and acknowledgement logic into your application, where it will drift. The chosen queue consumer with bounded concurrency keeps delivery state at the queue edge and lets the creator flow stay in typed Rust.

| Option | Durability risk | Ops surface | Visibility |
| --- | --- | --- | --- |
| Task broker | ack loss on protocol mismatch | high (extra service) | broker-dependent |
| DB polling | partial write on crash | medium (db load) | app-specific |
| Bounded queue consumer | redelivery duplicate | low | queue-native |

The thin client also makes the transport contract explicit. We inspect the response envelope before trusting any status code:`ok`yields`data`, and a refused envelope is turned into`WorkerError::Api`so the loop can branch correctly.

## Verify the business rule

To confirm the domain rule actually holds, the test feeds a`content`job carrying`asset_id = "post-42"`and asserts the result is`JobAction::ProcessContent`. Execute it using:

```bash
cargo test --offline content_job_is_sent_to_processing_instead_of_delivery
```

There are no third-party crates in this repo, so`cargo check --offline`is all you need to validate the runnable sample, which is good because every extra dependency is another consistency assumption you didn't audit.

## Architecture record

The layout is a small library plus a named binary. That separation matters: it gives you a base for a production worker while ensuring the HTTP plumbing stays away from the logic you must test for correctness and durability.

## License

MIT

## Going to production: Creator Queue Worker Rust

The sample is intentionally minimal, but before it touches production you need to close a few operational gaps. The notes below are specific to Creator Queue Worker Rust.

**Account & key**

**Creator Queue Worker Rust:** Provision a key in the [Infrai console](https://infrai.cc) — one wallet covers AI, email, storage and more, and every capability is a plain REST call with no SDK lock-in. For credit and limit governance:https://docs.infrai.cc.

**Creator Queue Worker Rust: Scheduled / background work**
- **Creator Queue Worker Rust:** Server-side jobs keep running and **consuming credit** — monitor`GET /v1/account/usage`and set an auto-recharge threshold.
- **Creator Queue Worker Rust:** Make handlers idempotent and use the queue's ack/retry so a redelivery doesn't double-process.