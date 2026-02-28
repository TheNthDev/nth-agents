# ZeroClaw + Actix Cluster Sharded Agents

This project implements a scalable web application that provides users with their own dedicated AI agents. It leverages the **Actix Actor Framework** for concurrency and the **ZeroClaw** library for AI assistant capabilities, with **Cluster Sharding** via `actix-telepathy` to scale across multiple nodes.

## Architecture

### 1. User Agent Actors
Each user is represented by a `UserAgentActor` (`src/actor.rs`). This actor:
- Encapsulates a `zeroclaw::Agent`.
- Maintains user-specific state and history.
- Processes messages asynchronously.
- Logs every interaction with an `[EVENT_LOG]` prefix for auditing and event-sourcing potential.

### 2. Cluster Sharding
The application uses `actix-telepathy` to enable cluster-wide communication:
- Actors can be addressed across different physical nodes.
- The `Cluster` actor handles node discovery and heartbeat.
- `RemoteMessage` implementations allow transparent routing of user requests to the node where their actor resides.

### 3. Web API
An **Actix-web** server (`src/main.rs`) provides the entry point for users:
- `POST /agent/{user_id}/turn`: Sends a message to a specific user's agent.
- Routes requests to the appropriate `UserAgentActor` address.

## ZeroClaw Integration

The project integrates `zeroclaw` as a core dependency. The `UserAgentActor` is responsible for:
- Initializing the agent with the required providers (e.g., OpenAI, Anthropic). Supports `AGENT_PROVIDER` and `AGENT_MODEL` environment variables.
- Managing the `AgentTurn` lifecycle.
- Handling responses and tool calls from the LLM.
- Implementing custom tools (e.g., `WeatherTool`) to extend agent capabilities. The `WeatherTool` supports real-time data via the OpenWeatherMap API when `OPENWEATHERMAP_API_KEY` is set.

## Scaling Strategy

To scale the system:
1. **Deploy multiple instances**: Run the binary on different servers.
2. **Seed Nodes**: Provide seed node addresses in the `Cluster` configuration to allow instances to find each other.
3. **Transparent Routing**: `actix-telepathy` ensures that even if a request hits Node A, but the user's actor is on Node B, the message is routed correctly.

## Getting Started

### Prerequisites
- Rust 1.75+ (Edition 2024 used)
- Access to an LLM provider (configured via ZeroClaw)

### Running Locally
```bash
cargo run
```

### Testing
```bash
curl -X POST http://localhost:8087/agent/user123/turn \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello, agent!"}'
```

## Future Roadmap
- [x] Implement persistent event-logs for state reconstruction.
- [x] Add support for custom ZeroClaw tools per user.
- [x] Integrate full Cluster Registry for dynamic actor migration.
- [x] Create a Web UI for agent interaction and signup.
