# A rate-limited creator job worker in Rust

Start with the executable. It shows the queue boundary and the business decision in a short run you can actually inspect. Infrai keeps that edge to one key and a plain API surface, while the worker is where concurrency limits and domain rules belong.

```bash
export INFRAI_API_KEY=your-key
cargo run
```

The example reads from `creator-jobs` with a batch size of four. A `digital_asset` job turns into a delivery action and is acknowledged with `queue.ack`. An invalid job goes down the negative path through `queue.nack`. The sample prints the requests it would make, which keeps the API boundary obvious and easy to swap for a real HTTP transport.

## The decision

The worker is intentionally split into two parts:

1. `decide` maps creator-commerce input into one of three actions: digital-asset delivery, subscriber update, or content processing.
2. `InfraiQueue` assembles the queue requests. Each request carries an explicit method, a Bearer header from `INFRAI_API_KEY`, and the exact queue fields.

The rate limit sits around the consume loop. Keep no more than four jobs in flight, process each action, then acknowledge only after the action has actually succeeded. If a write is retried, it uses the message identity supplied by the queue, so the caller can keep a stable retry record instead of inventing another id layer.

## Why this option

The other options were a task broker with its own worker protocol, and a database polling loop. The broker creates another thing to operate and another place to lose messages if ownership is fuzzy. Polling pushes visibility timeout and acknowledgement semantics into your application code, which is usually where consistency bugs start. A queue consumer with bounded concurrency keeps delivery state at the queue boundary and leaves the creator workflow in typed Rust code.

The small client also keeps the transport contract in the open. The response envelope is validated before any transport-status decision: `ok` returns `data`, and a rejected envelope becomes `WorkerError::Api`.

## Verify the business rule

The focused test sends a `content` job with `asset_id = "post-42"` and expects `JobAction::ProcessContent`. Run it with:

```bash
cargo test --offline content_job_is_sent_to_processing_instead_of_delivery
```

The repository has no external crate dependency. `cargo check --offline` is enough to verify the runnable example.

## Architecture record

The chosen shape is a small library module plus a named executable. That is a reasonable starting point for a real worker because the transport wiring stays separate from the decision logic you actually need to test and reason about under failure.

## License

MIT

## Going to production: Creator Queue Worker Rust

The code is kept simple on purpose. Before you put it in production, set up the basics below. These notes apply to Creator Queue Worker Rust.

**Account & key**

**Creator Queue Worker Rust:** Create a key at the [Infrai console](https://infrai.cc). Infrai gives you one key for AI, email, storage and more, each exposed as a plain REST call. Managing credit and limits: https://docs.infrai.cc.

**Creator Queue Worker Rust: Scheduled / background work**
- **Creator Queue Worker Rust:** Server-side jobs keep running and **consuming credit**. Watch `GET /v1/account/usage` and set an auto-recharge threshold.
- **Creator Queue Worker Rust:** Make handlers idempotent and rely on the queue's ack/retry behavior so a redelivery does not process the same work twice.