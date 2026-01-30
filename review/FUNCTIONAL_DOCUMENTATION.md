# Maiko Functional Documentation

## Overview
Maiko is a lightweight actor runtime for Tokio that implements a topic-based publish/subscribe model. Unlike traditional actor systems that rely on direct addressing, Maiko actors communicate by emitting events to topics. The runtime automatically routes these events to all actors subscribed to the corresponding topic.

## Core Functional Areas

### 1. Messaging System
The fundamental unit of communication in Maiko is the **Event**.

*   **Events**: Strongly-typed Rust enums or structs that implement the `Event` trait. They represent facts or commands within the system. events are cloneable and thread-safe (Send + Sync).
*   **Envelopes**: All events are wrapped in an `Envelope` before delivery. The envelope provides metadata alongside the event payload:
    *   **Sender Identity**: The `ActorId` of the actor that produced the event.
    *   **Correlation ID**: A unique identifier to track causality chains across multiple asynchronous steps.
    *   **Event ID**: A unique identifier for the specific event instance.

### 2. Topic-Based Routing
Routing is decoupled from actor identity.

*   **Topics**: Types that implement the `Topic` trait map specific events to logical channels.
*   **Subscription**: Actors declare their interest in specific topics upon registration.
*   **Routing Logic**: When an event is published, Maiko uses the `Topic::from_event()` implementation to determine the destination topic and delivers the event to all subscribers of that topic.
*   **DefaultTopic**: A built-in topic for simple broadcast scenarios where all actors receive all events.

### 3. Actor Model
Actors are the execution units of the system.

*   **State Isolation**: Each actor maintains its own private state, accessible only via its `handle_event` method.
*   **Sequential Processing**: Events are processed one at a time per actor, eliminating race conditions within a single actor's state.
*   **Capabilities**:
    *   **`handle_event`**: The primary callback for processing incoming messages.
    *   **`step`**: A periodic hook that allows actors to perform background work, emit events on a timer, or poll external resources without blocking the event loop.
    *   **Lifecycle Hooks**: Optional `on_start`, `on_shutdown`, and `on_error` methods to manage initialization and cleanup.

### 4. Orchestration & Runtime
The **Supervisor** is the entry point and manager of the Maiko runtime.

*   **Registration**: The Supervisor provides APIs (`add_actor`) to instantiate actors and bind them to topics.
*   **Life Cycle Management**: reliable starting, stopping, and graceful shutdown of the actor system.
*   **Execution Control**: Supports both non-blocking execution (`start`) and blocking execution (`run`/`join`) to integrate with various main loop patterns.

## Auxiliary Functions

### Testing Harness
Maiko includes a specialized `Harness` for deterministic testing of concurrent actor flows.

*   **Event Recording**: Captures the stream of events flowing through the system.
*   **Assertions**: verifiable checks for event delivery ("Did Actor A receive Event X from Actor B?").
*   **Spies**: Inspect internal state or event history without modifying actor code.

### Monitoring
The system exposes a `Monitor` trait to tap into the low-level event lifecycle.

*   **Hooks**: `on_event_sent`, `on_event_handled`, `on_actor_error`.
*   **Use Cases**: This extension point is designed for logging, metrics collection (e.g., Prometheus), and distributed tracing integration.
