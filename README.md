# Thread Pool from Scratch

A minimal, zero-dependency thread pool implemented in Rust using only the standard library. Built to understand the mechanics behind worker threads, shared channels, and graceful shutdown.

## Overview

This project implements a fixed-size thread pool that:

- Spawns a configurable number of OS threads at startup
- Distributes jobs across workers via a single shared MPSC channel
- Shuts down gracefully when dropped — no threads are leaked

There are no external crates. Every primitive (`Arc`, `Mutex`, `mpsc`, `JoinHandle`) comes from `std`.

## How It Works

```
                    ┌──────────────────────────────────┐
                    │           ThreadPool             │
                    │                                  │
 pool.execute(f) ──►│  sender ──► channel ──► receiver │
                    │                         (shared) │
                    │  Worker 0 ◄──────────────────────┤
                    │  Worker 1 ◄──────────────────────┤
                    │  Worker 2 ◄──────────────────────┤
                    │  Worker 3 ◄──────────────────────┘
                    └──────────────────────────────────┘
```

### Components

**`Job`** — a type alias for a heap-allocated, once-callable closure:
```rust
type Job = Box<dyn FnOnce() + Send + 'static>;
```

**`Worker`** — owns a thread that loops forever, pulling jobs off the shared receiver:
```rust
loop {
    let message = receiver.lock().unwrap().recv();
    match message {
        Ok(job) => job(),
        Err(_)  => break,   // channel closed → exit
    }
}
```

**`ThreadPool`** — holds the sender side of the channel and a `Vec<Worker>`. Calling `execute` boxes the closure and sends it down the channel. One of the idle workers picks it up and runs it.

### Graceful Shutdown via `Drop`

When `ThreadPool` goes out of scope, its `Drop` implementation:

1. Drops the `sender` — this closes the channel
2. Workers' `.recv()` calls return `Err` — each worker exits its loop
3. `thread.join()` is called on each worker thread — the main thread waits for all of them to finish

No sentinel values, no `AtomicBool` flags. The channel itself signals shutdown.

### Shared Receiver

A single `mpsc::Receiver` cannot be cloned, so it is wrapped in `Arc<Mutex<Receiver<Job>>>` and an `Arc::clone` is handed to each worker. Workers compete for the lock; whoever acquires it first pulls the next job. This is the standard "work-stealing via mutex" pattern.

## Project Structure

```
src/
└── main.rs    # ThreadPool, Worker, Job, and a demo main()
Cargo.toml
```

## Running

```bash
cargo run
```

Example output (thread IDs and order will vary):

```
Job 0 running on thread ThreadId(2)
Job 1 running on thread ThreadId(3)
Job 2 running on thread ThreadId(4)
Job 3 running on thread ThreadId(5)
Job 4 running on thread ThreadId(2)
Job 5 running on thread ThreadId(3)
Job 6 running on thread ThreadId(4)
Job 7 running on thread ThreadId(5)
Shutting down worker 0
Shutting down worker 1
Shutting down worker 2
Shutting down worker 3
```

8 jobs spread across 4 workers. The "Shutting down" lines come from the `Drop` impl as the pool exits.

## Key Rust Concepts Demonstrated

| Concept | Where |
|---|---|
| `Arc<Mutex<T>>` for shared mutable state across threads | Shared receiver |
| `mpsc` channel for work distribution | `ThreadPool::execute` → `Worker` |
| `Box<dyn FnOnce() + Send + 'static>` for type-erased closures | `Job` type alias |
| `Option<T>` + `.take()` for one-shot ownership transfer | `sender` and `thread` fields |
| `Drop` for deterministic resource cleanup | `impl Drop for ThreadPool` |
| `thread::JoinHandle` for waiting on thread completion | `Worker::thread` |

## Limitations

This is an educational implementation. Production thread pools would additionally handle:

- **Panicking jobs** — a panicking closure poisons the `Mutex`. The current code would propagate the panic. A robust pool would catch panics with `std::panic::catch_unwind`.
- **Dynamic pool resizing** — the pool size is fixed at construction time.
- **Job prioritization** — all jobs are treated equally; there is no priority queue.
- **Backpressure** — `execute` sends to an unbounded channel; callers are never blocked regardless of how many jobs are queued.

## References

- [The Rust Book — Chapter 20: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html) — the canonical introduction to this pattern
- [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html)
- [`std::sync::Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
