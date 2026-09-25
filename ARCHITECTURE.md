# Erk Engine Architecture

> Status: early design. This document describes the target architecture; most of it is not
> implemented yet. Update it as the design evolves.

Erk is an embeddable browser engine written in Rust, using WGPU for hardware-accelerated
rendering and a multi-process architecture for security and stability.

## Process model

```
┌──────────────────────────────┐
│  Shell / Broker (erk-shell)  │  privileged: OS window, input, process lifecycle
└──────┬───────────────┬───────┘
       │ IPC           │ IPC
┌──────▼───────┐ ┌─────▼────────┐
│  Renderer    │ │  Network     │
│ (sandboxed)  │ │  process     │
│ erk-renderer │ │ erk-network  │
│   erk-dom    │ └──────────────┘
└──────────────┘
```

- **Shell / Broker** — the only privileged process. Owns the OS window (winit), routes
  input, spawns and supervises the other processes, and brokers every request that needs
  OS access.
- **Renderer** — one or more sandboxed processes. Parses HTML/CSS, builds the DOM, runs
  style, layout and painting, and hosts the JavaScript engine. It has no direct access to
  the file system or network.
- **Network** — performs HTTP requests and enforces Fetch rules (CORS, redirects, caching)
  on behalf of renderers.

A crash in a renderer must never bring down the shell.

## Crates

| Crate          | Responsibility                                   |
|----------------|--------------------------------------------------|
| `erk-shell`    | OS window (winit), broker, process management    |
| `erk-renderer` | HTML/CSS pipeline: style, layout, paint (WGPU)   |
| `erk-network`  | Networking and the Fetch standard                |
| `erk-dom`      | DOM tree storage and manipulation                |

## IPC

Processes communicate through message passing. Messages are typed Rust structs that are
serialized at the process boundary. Every message received by the broker from a renderer is
treated as untrusted input and validated.

## JavaScript integration

The JavaScript engine is implemented in C++ and embedded in the renderer process. The DOM
lives in Rust (`erk-dom`) and is exposed to JavaScript through a bindings layer. Keeping the
unsafe FFI surface small and well-audited is a core design goal.

## Rendering pipeline

```
HTML → Parse → DOM → Style → Layout → Paint → Composite (WGPU)
```

## Conformance

Correctness is measured against the [Web Platform Tests](https://web-platform-tests.org/)
(WPT). WPT runs will be added to CI.
