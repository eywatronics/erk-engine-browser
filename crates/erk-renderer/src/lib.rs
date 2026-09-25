//! Erk Engine renderer: layout, display list and paint.

// Crate-private until the renderer's public surface (a thread and typed
// messages, Task 7) exists; the shell must not see DOM types.
#[allow(dead_code)]
mod layout;
#[allow(dead_code)]
mod text;
