You are assisting with Asteria, a Rust-based client-server system designed to relay keyboard and mouse input from a Linux (Hyprland/Wayland) machine to a Windows 11 machine over the network. The system is designed for low-latency, secure, and modular remote input control.

## Project Context

- Client: Runs on Linux (Wayland/Hyprland)
  - Captures keyboard and mouse events from `/dev/input/event*` using `libinput`
  - Sends serialized input events over TCP using `tokio`
  - Requires root privileges for low-level device access

- Server: Runs on Windows 11
  - Receives input events
  - Translates and simulates input using the `enigo` crate
  - Must run as Administrator

- Transport:
  - TCP preferred for minimal overhead

- Serialization:
  - Use `serde` with `bincode` for compact binary communication

## Code Style and Architecture Guidelines

You are an expert in Rust, async programming, and concurrent systems. Follow these principles when generating suggestions.

### Key Principles

- Write clear, concise, and idiomatic Rust code with accurate examples
- Use expressive variable names that convey intent (e.g., `is_ready`, `has_data`)
- Follow Rust naming conventions: `snake_case` for variables/functions, `PascalCase` for types
- Avoid duplication by encapsulating reusable logic in modules and functions
- Prioritize modularity, maintainability, and efficient resource usage
- Ensure safety, concurrency, and performance by leveraging Rust’s type and ownership system

### Async Programming

- Use `tokio` as the asynchronous runtime
- Implement async functions using `async fn` and spawn tasks with `tokio::spawn`
- Use `tokio::select!` for managing concurrent async tasks
- Prefer structured concurrency: use scoped tasks and clean cancellation strategies
- Implement retries, backoffs, and timeouts using `tokio::time` utilities
- Avoid blocking inside async functions; offload using `spawn_blocking` when necessary

### Error Handling and Safety

- Utilize `anyhow` for error handling
- Use `Result` and `Option` for safe error handling
- Use the `?` operator for propagating errors in async functions
- Await only at safe yield points and handle task cancellations properly

### Performance Optimization

- Use async only when necessary; avoid overhead in purely synchronous paths
- Avoid blocking operations in async contexts; use `spawn_blocking` when required
- Use `tokio::task::yield_now` to yield control cooperatively
- Optimize locking granularity and reduce contention across async tasks

### Prioritize:

- Platform-specific input capture (Linux) and simulation (Windows)
- Efficient, robust async networking using `tokio`
- Serialization with `serde` and `bincode`
- Modular and testable code design

### Avoid:

- X11-based solutions (project targets Wayland)
- GUI automation frameworks like `xdotool` or `autohotkey`
- Browser-based solutions unless explicitly required
