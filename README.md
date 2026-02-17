middleground
------------

**A platform for constructive dialogue across ideological divides.**

Middleground is a real-time messaging platform designed to mitigate the affective polarization and spread of misinformation caused by echo chambers and engagement-farming prevalent on modern social media.
By facilitating thoughtful, evidence-oriented one-on-one conversations conversations between people with differing viewpoints, Middleground aims to create an environment where genuine understanding can emerge.

[![Rust](https://img.shields.io/badge/rust-1.88%2B-red?logo=rust&style=for-the-badge)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL-purple?style=for-the-badge)](LICENSE)

### Quick Links

* [Core Features](#core-features)
* [Architecture](#architecture)
* [Tech Stack](#technology-stack)
* [API Endpoints](#api-endpoints)
* [Getting Started](#getting-started)
* [Project Status](#project-status)
* [Further Docs](#further-documentation)
* [Acknowledgements](#acknowledgements)

### Vision

Most online discourse suffers from socio-psychological effects like group polarization and social identity, where like-minded individuals reinforce each other's views in increasingly extreme directions.
Middleground addresses this by:

- **Intelligent Matchmaking**: Pairing users based on conversation prompts and past interaction quality
- **Quality Moderation**: Using analytics and user reports to maintain constructive dialogue standards
- **Source-First Discussions**: Making evidence sharing seamless with citation tools and credibility ratings
- **Identity Separation**: Allowing the exploration of ideas without attachment to personal identity
- **One-on-One Dialogue**: Reducing the normative and informational influence of group membership

The platform encourages users to engage with challenging ideas and diverse perspectives in a structured environment that rewards thoughtful, empathentic conversation rather than performative outrage.

### Core Features
#### 1. Matchmaking

Users describe topics they want to discuss via a prompt. 
The system matches them with conversation partners based on:
- Topic relevance and complementary perspectives (you should match with someone with similar interests, but different perspectives)
- Conversation quality "ELO" scores from past interactions (if you're doing your best, you deserve to talk to someone doing the same)

This creates balanced pairings where both parties are genuinely interested in the topic and have demonstrated capacity for constructive dialogue.

#### 2. Moderation

A robust reputation system maintains platform quailty:
- User reports verified through conversation history analysis
- ELO scores that reflect conversational quality over time
- Grace periods for new users paired with experienced, kind conversationalists
- 30-day conversation retention for moderation review

#### 3. Source Validation

Built-in tools make evidence-based discussion frictionless:
- Easy citation sharing with automatic metadata extraction
- Source credibility ratings and publication details
- Reference management for saving and reviewing shared sources
- Encourages substantive debate grounded in verifiable interaction

#### 4. Real-Time Messaging

Production-grade WebSocket infrastructure, supporting:
- Sending, replying, editing, and deleting messages
- Full edit history preservation
- Horizontal scalability via Redis pub/sub

### Architecture

Middleground uses a *microservices-inspired monorepo* structure with clear separation of concerns.
Each crate handles a specific domain while sharing common infrastructure through workspace dependencies.

```
┌───────────────────────────────────────────────────────────────────┐
│                          Client Layer                             │
│                      (Web/Mobile Frontend)                        │
└───────────────────────────────┬───────────────────────────────────┘
                                │ HTTP/WebSocket
                                ▼
┌───────────────────────────────────────────────────────────────────┐
│                        API Gateway                                │
│   ┌────────────┐  ┌──────────────┐  ┌─────────────────────────┐   │
│   │   Auth     │  │  WebSocket   │  │   Future Services       │   │
│   │  Service   │  │   Service    │  │  (Matchmaking, etc.)    │   │
│   └────────────┘  └──────────────┘  └─────────────────────────┘   │
└───────────────────────────────┬───────────────────────────────────┘
                                │
                   ┌────────────┼────────────┐
                   ▼            ▼            ▼
           ┌──────────┐   ┌───────────┐   ┌──────────┐
           │PostgreSQL│   │   Redis   │   │ Supabase │
           │(Supabase)│   │ (Pub/Sub) │   │  (Auth)  │
           └──────────┘   └───────────┘   └──────────┘
```

#### Crate Structure

| Crate | Purpose | Key Technologies |
|-------|---------|------------------|
| **`api_gateway`** | Main entry point; composes all services into a unified HTTP server | Axum, Tower |
| **`auth`** | JWT-based authentication with OTP verification and JWKS support | Supabase Auth, jsonwebtoken |
| **`ws`** | Real-time WebSocket messaging with full-duplex communication | Axum WebSocket, Redis Pub/Sub |
| **`db`** | Database abstraction layer with connection pooling | SQLx, PostgreSQL |
| **`shared`** | Common types and application state shared across services | - |
| **`cli`** | Command-line testing tool for backend validation | Tokio, Tungstenite |
| **`source_validation`** | Citation and source credibility tools *(in development)* | - |

Additional crates will be introduced as development progresses.

**Separation of Concerns.** 
Each crate has a single, well-defined responsibility.
The `api_gateway` orchestrates without implementing business logic.
Services like `auth` and `ws` are self-contained, and can be extracted into standalone services if desired.

**Database-First Consistency.**
All changes are persisted into PostgreSQL prior to being broadcast.
For instance, messages sent via WebSocket connections, once received, are stored prior to broadcasting via Redis.
This has valueable guarantees: an event occurs if and only if it has been recorded in the database.
The database serves as a source of truth.

**Horizontal Scalability.**
Multiple `api_gateway` instances can run simultaneously.
Redis pub/sub ensures messages published by one instance reach clients connected to other instances.
The database connection pool prevents resource exhaustion.

### Technology Stack

**Core:**
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit

**Infrastructure:**
- [PostgreSQL](https://www.postgresql.org/) (via Supabase) - Primary database
- [Supabase](https://supabase.com/) - Authentication backend
- [Redis](https://redis.io/) - Pub/sub messaging and caching

**Development:**
- [Docker](https://www.docker.com/) - Containerization
- [Cargo](https://doc.rust-lang.org/cargo/) - Build system and package manager

### Getting Started

#### Prerequisites

- **Rust** 1.85 or higher
- **PostgreSQL** database (Supabase recommended)
- **Docker & Docker Compose** for Redis and container orchestration (recommended)

It is recommended that Supabase is used as the PostgreSQL database provider.
The project's authentication crate has a default implementation relying on Supabase authentication services.
However, an alternative database can be used if an authenticator is implemented for it.
More information about this can be found [here](auth/README.md).

#### Quick Start with Docker

1. Clone the repository
```bash
git clone https://github.com/barnaste/middleground.git
cd middleground
```

2. Configure environment variables
```bash
cp .env.example .env
# Edit .env with your credentials and database URL
```

3. Generate SQLx query cache (first time only)
```bash
cd db

# Install SQLx CLI if you haven't already
cargo install sqlx-cli --no-default-features --features postgres

# Generate the query cache for offline Docker builds
cargo sqlx prepare
```
This creates a `.sqlx/` directory in the `db` crate that should be committed to version control.
If you are not writing additional queries in `db`, you may skip this step: the `.sqlx/` directory you cloned will suffice.

4. Launch the services
```bash
docker compose up
```
The backend will be available at `http://localhost:8080`

5. Test with the CLI
```bash
cargo run -p cli -- --host http://localhost:8080 --username your@email.com
```

#### Configuration
The following environment variables require configuration in step (2):

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host:5432/db` |
| `REDIS_URL` | Redis connection string | `redis://redis:6379` |
| `SUPABASE_URL` | Supabase project URL | `https://xyz.supabase.co` |
| `SUPABASE_API_KEY` | Supabase API key | `eyJhbG...` |
| `RUST_LOG` | Logging level (optional) | `info,api_gateway=debug` |

The `RUST_LOG` environment variable controls the verbosity of the backend's tracing logs.
These logs are visible in the Docker Compose output, and can be viewed after detaching with `docker compose logs -f`.
By default, they are set to `info` level for all dependencies, and `debug` level for application crates.

**Important**: When running in Docker Compose, use the service name `redis` as the hostname 
```bash
REDIS_URL=redis://redis:6379
```
For local development outside Docker, set the service name to `localhost` instead:
```bash
REDIS_URL=redis://localhost:6379
```

### API Endpoints

#### Authentication
- `POST /auth/send-otp` - Request an OTP code via email
- `POST /auth/verify-otp` - Verify OTP and receive access/refresh tokens
- `POST /auth/refresh` - Refresh an expired access token
- `POST /auth/logout` - Invalidate current session

#### WebSocket
- `ANY /ws?conversation_id=<uuid>` - Upgrade to WebSocket for real-time messaging

*Authentication*: All WebSocket connections require a valid JWT in the `Authorization` header.

### Project Status

#### Complete
- [x] JWT authentication with Supabase and JWKS
- [x] Real-time WebSocket messaging
- [x] Horizontal server scalability via Redis
- [x] CLI testing tool
- [x] Docker Compose deployment

#### In Progress
- [ ] Matchmaking service with ELO-based pairing
- [ ] Moderation system with reputation tracking
- [ ] Source validation and citation tools
- [ ] Frontend (web/mobile)

#### Planned
- [ ] Conversation analytics and insights
- [ ] User preference learning
- [ ] Image sharing support
- [ ] Conversation archival and export

### Further Documentation

* [Authentication](auth/README.md)
* [WebSocket Service](ws/README.md)
* [CLI Tool](cli/README.md)

### License

This project is licensed under the GNU Affero General Public License. See [LICENSE](LICENSE) for details.

### Acknowledgements

Built with Rust's powerful type system and async ecosystem.
Special thanks to:
- The Tokio team for building robust async infrastructure
- The Axum maintainers for a delightful web framework
- The Supabase team for authentication and database tooling
- All contributers to the Rust ecosystem
