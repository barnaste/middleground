ws
----

A production-ready WebSocket service for real-time messaging built on axum and Redis Pub/Sub.

**ws** provides complete WebSocket infrastructure for Middleground's real-time messaging system.
It handles bidirectional communication between clients and the server, message persistence, and real-time broadcasting via Redis pub/sub.
The architecture ensures messages are committed to the database prior to being broadcast, providing consistency guarantees.

[![Rust](https://img.shields.io/badge/rust-1.87%2B-red?logo=rust&style=for-the-badge)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL-purple?style=for-the-badge)](../LICENSE)

### Quick Start

Integrating the WebSocket service into your application requires wrapping it with authentication middleware.
The service expects the user's UUID to be injected into request extensions by the auth layer.

```rust
use ws::router as ws_router;
use shared::AppState;
use axum::Router;

let state = AppState {
    db_pool: todo!(),
    redis: todo!(),
};

let app = Router::new()
    .merge(ws_router(state))
    .layer(/* insert your user-ID-injecting middleware here... */);
```

### Architecture

The service follows a clean architecture with clear separation of concerns.
When a client connects, the handler validates their access to the conversation before upgrading the HTTP connection to WebSocket.
Once established, the session manager splits the socket into read and write halves, enabling full-duplex communication without blocking.

```
 ┌───────────────────────────────────────────────────────────┐
 │                     Client WebSocket                      │
 └────────────────────────────┬──────────────────────────────┘
                              │                               
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Session Handler                        │
│    ┌────────────────┐                ┌────────────────┐     │
│    │ Read Task      │                │ Write Task     │     │
│    │ Client → DB    │                │ Redis → Client │     │
│    │ Client → Redis │                │                │     │
│    └────────────────┘                └────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

The read task processes incoming messages from the client, validates them, persists them to the database, and publishes them to Redis.
The write task subscribes to the conversation's Redis channel and forwards all published messages to the client.
This asynchronous design allows messages to be sent and received simultaneously.

#### Module Organization

This crate is organized into modules:
- `handler` serves as the entry point for WebSocket connections, performing authorization checks and upgrading HTTP connections;
- `session` manages the lifecycle of individual WebSocket connections, coordinating the flow of messages between client, database, and Redis;
- `messages` defines the message types and implements the business logic for processing each operation; and
- `error` provides type-safe error handling with proper HTTP status code mapping.

### Message Protocol

All messages use JSON with tagged union serialization. 
Field names are camelCase, while Rust code uses snake_case internally.
Serde handles this conversion automatically.

Clients send three types of messages.
To create a new message, they send a payload containing the text content and an optional UUID of a message being quoted.
To edit an existing message, they provide the message UUID and new content.
To delete a message, they provide only the UUID.
All operations are validated against the database to ensure users can only modify their own messages.

```json
{
    "type": "send",
    "payload": {
        "content": "Hello, world!",
        "quotedId": null
    }
}

{
    "type": "edit",
    "payload": {
        "messageId": null,
        "content": "Goodbye, world!"
    }
}

{
    "type": "delete",
    "payload": {
        "messageId": null,
    }
}
```

The server broadcasts messages to all participants in a standardized format that includes the message ID, sender ID, content, and a server-generated timestamp.
Using server-side timestamps prevents client tampering and ensures consistent ordering across all clients.
The requests made by the examples above would broadcast something of the form below.

```json
{
    "type": "send",
    "message_id": "550e8400-e29b-41b4-a716-446655440000",
    "senderId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "content": "Hello, world!",
    "quotedId": null,
    "timestamp": "2024-02-11T15:30:00Z"
}

{
    "type": "edit",
    "message_id": "550e8400-e29b-41b4-a716-446655440000",
    "senderId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "content": "Goodbye, world!",
    "timestamp": "2024-02-11T15:31:00Z"
}

{
    "type": "delete",
    "message_id": "550e8400-e29b-41b4-a716-446655440000",
    "senderId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "timestamp": "2024-02-11T15:32:00Z"
}
```

### Database Schema

The service is database-agnostic, but expects a database interface that supports `create_message`, `edit_message`, and `soft_delete_message` functions in a `db::queries::messages` module, and a `user_has_access` function in a `db::queries::conversations` module.

In practise, these functions use a two-table schema that supports both message persistence and edit history.
The `message` table stores core message metadata including the conversation ID, sender ID, creation timestamp, optional quoted message reference, and a soft delete flag.
The `message_atom` table stores the actual content with a revision number, allowing the system to preserve the full edit history of each message.

When a message is edited, a new atom is created with an incremented revision number while the original content remains in the database.
This design enables features like "view edit history" while maintaining referential integrity for quoted messages.
Soft deletes mark messages as deleted without removing them from the database, preserving conversation contet and supporting potential moderation workflows.

### Redis Integration

Each conversation has a dedicated Redis pub/sub channel named `conversation:{conversation_id}`. 
When a message is successfully persisted to the database, it is published to this channel.
All WebSocket sessions subscribed to the channel receive the message and forward it to their respective clients.

Thes ervice uses RESP3 (Redis Serialization Protocol 3) instead of the older RESP2 protocol.
RESP3 provides push-based notifications, meaning the Redis server can proactively send messages to subscribed clients without polling.
This reduces latency and improves efficiency, which is critical for real-time messaging applications.

### Performance

The concurrency model is designed for maximal throughput.
Each WebSocket session spawns two independent tasks that run concurrently, one handling incoming messages from the client (the read task) and the other forwards messages from Redis to the client (the write task).
Because they operate on separate halves of the split socket, they never block one another.

The architecture scales horizontally without modification.
Multiple server instances can run simulatneously, each of which handling its own set of WebSocket connections.
Redis pub/sub acts as the message bus, ensuring that a message published by one server instance reaches clients connected to other instances.
The database connection pool prevents exhaustion under high load, and each WebSocket maintains its own Redis connection to avoid contention.

### Error Handling

The service uses a comprehensive error type that covers all failure scenarios and maps them to appropriate HTTP status codes.
Unauthorized access returns 401, whereas database errors, Redis failures, and WebSocket protocol errors all return 500.
Every error is logged with structured context including user ID, conversation ID, and the specific operation that failed.

Crucially, individual message processing errors do not terminate the WebSocket connection.
If a client sends a malformed JSON message or tries to edit a message they do not own, the error is logged and the session continues.
This resilience allows connections to survive transient failures and client-side bugs without requiring reconnection.

### Troubleshooting

**WebSocket connections fail immediately:** This usually means either that the JWT is invalid/expired, or the user doesn't have access to the conversation.
Check that authentication middleware is properly configured and that the conversation participant records exist in the database.

**Messages aren't being broadcast:** Verify Redis is running and accessible with `redis-cli ping`. 
Check the application tracing logs for Redis connection errors.
Ensure RESP3 protocol is configured correctly in the Redis instance.

**Database errors during message operations:** These typically indicate schema mismatches or constraint violations.
Verify the `message` and `message_atom` tables match the expected structure.
Look for foreign key violations in the logs if messages reference non-existent conversations or users.

### Dependencies

**Core functionality:**
- `axum` (workspace, with ws features) - Web framework for WebSocket handling and HTTP upgrade
- `futures` - Async stream utilities for WebSocket message handling
- `tokio` (workspace) - Async runtime for concurrent read/write tasks

**Data persistence and messaging:**
- `sqlx` (workspace) - PostgreSQL database client for message persistence
- `redis` (workspace, with tokio-comp features) - Redis client for pub/sub messaging with RESP3 support
- `db` (path dependency) - Database abstraction layer for message operations
- `shared` (path dependency) - Shared types and application state

**Serialization and types:**
- `serde` / `serde_json` (workspace) - JSON message protocol serialization
- `uuid` (workspace) - Message and conversation identifier types
- `chrono` (workspace) - Timestamp generation for server-side message ordering

**Error handling and logging:**
- `thiserror` (workspace) - Structured error types with HTTP status code mapping
- `tracing` - Structured logging for error context and debugging

### License
Please see the workspace root for license information.
