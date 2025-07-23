## Tick-Driven Approach

The current approach uses tick intervals to sleep until events or input are
available. Using an async task manager, none of the asynchronous commands are
ever blocking the main loop, and the IO processing can still be synchronously
polled while "awake" for immediate responsiveness.

1. Eliminated Busy-Wait Loop

- Before: tokio::task::yield_now().await in a tight loop
- After: Event-driven loop that only wakes up when:
  - Async tasks complete
  - User input events occur
  - Periodic tick timer fires (60 FPS for cleanup/rendering)

2. Render-Only-When-Needed

- Before: Rendered on every loop iteration
- After: Added needs_render flag that only triggers renders when:
  - Model state changes from messages
  - Async operations complete
  - First render on startup

3. Optimized Event Processing

- Before: Sequential polling with timeouts
- After: Immediate processing of all available events, then wait
  - Process async task completions (non-blocking)
  - Process input events (non-blocking)
  - Only wait when no events are pending

4. Smart Loop Control

- When events are available: Process immediately and continue
- When no events: Wait for tick timer using tokio::select!
- Periodic cleanup and rendering at 60 FPS max

Performance Benefits:

The implementation now properly balances responsiveness with efficiency - it responds instantly to events while using
minimal CPU when idle, making it suitable for production use without the performance issues of the previous busy-wait
approach.


## Fully Async Approach

This could be further extended to use a fully asynchronous approach, where the
main thread follows the the `select!` behavior and sleeps until events arrive.

This approach is not the current direction, because the channel communication
for all input handling is difficult to implement and maintain.

### 1. Requirements Satisfied

The fully asynchronous `select!` model is designed to meet the following critical performance and responsiveness
requirements:

• Zero CPU Usage When Idle: It eliminates the "busy-wait" loop, allowing the application to consume virtually no CPU
resources while waiting for user input or background task completions.
• Immediate UI Updates from Background Tasks: The UI will update the instant an asynchronous task (e.g., a network
request) completes, without waiting for the user to provide any input. This ensures the application feels highly
responsive and always reflects the true current state.
• Efficient Rendering: The view will be re-rendered only when the application's state actually changes in response
to an event. This prevents wasteful rendering cycles and reduces terminal output, contributing to a smoother
experience.

### 2. General Implementation Outline

The refactoring of the `run_async` function will proceed in four main steps:

1. Introduce a Channel for Async Tasks: The `AsyncTaskManager` will be modified to use a `tokio::sync::mpsc` channel.
Instead of being polled, it will now push Msg results from completed tasks directly into this channel.
2. Isolate Blocking Input Events: The synchronous poll_subscriptions function, which blocks while waiting for user
input, will be moved into its own dedicated thread using `tokio::task::spawn_blocking`. This task will use a second
mpsc channel to send user input events back to the main loop.
3. Rebuild the Main Loop with `tokio::select!`: The core loop in run_async will be replaced. The new loop will use a
`tokio::select!` macro to await messages from both the async task channel and the user input channel simultaneously.
The loop will only proceed when a message is received from either source.
4. Trigger Rendering on State Change: The `render_view()` function will be called from within the `select!` block,
immediately after a Msg has been received and processed by the update function. This guarantees that a render is
performed if and only if the model's state has changed.
