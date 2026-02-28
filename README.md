# ZeroClaw + Actix Cluster Sharded Agents

A distributed, scalable AI agent system built with Rust. This project leverages the **Actix Actor Framework** for concurrency, **actix-telepathy** for cluster sharding, and **ZeroClaw** for LLM-powered assistant capabilities.

## Features

- **Distributed Actors:** Transparent routing of user-specific agents across a multi-node cluster.
- **AI Integration:** Seamless integration with `zeroclaw` for LLM turns and tool calling.
- **Persistence:** Automatic state management and conversation history storage in `memory/`.
- **Web UI:** A built-in chat interface for interacting with agents.
- **Custom Tools:** Extensible tool system (e.g., includes a `WeatherTool`).
- **Observability:** Integrated logging with `[EVENT_LOG]` prefixes.

## Prerequisites

- **Rust:** 1.87+ (using Edition 2024)
- **API Key:** An OpenAI API key is recommended. If not provided, the system falls back to a mock/synthetic provider for development. For real-time weather data, an `OPENWEATHERMAP_API_KEY` is required.

## Getting Started

### 1. Clone the repository
```bash
git clone <repository-url>
cd agents
```

### 2. Configure Environment
Set your API keys by creating a `.env` file (see `.env.example` as a template):
```bash
cp .env.example .env
# Edit .env with your favorite editor and set your keys
```

Alternatively, you can export them directly:
```bash
export OPENAI_API_KEY=your_sk_key
export OPENWEATHERMAP_API_KEY=your_openweathermap_key
```

### 3. Run the Application

#### Single Node
```bash
cargo run
```
The Web UI will be available at `http://localhost:8087` (port may vary based on node address).

1.  Open the dashboard.
2.  Choose your **AI Provider** and **Model Name**.
3.  Select which **Tools** to enable (e.g., Weather Tool).
4.  Enter a **User ID** and click **Start Session**.

#### Cluster Mode
To run a two-node cluster:

**Node 1 (Seed Node):**
```bash
cargo run -- 127.0.0.1:1992
```

**Node 2:**
```bash
cargo run -- 127.0.0.1:1993 127.0.0.1:1992
```
Node 2 will join Node 1. Requests sent to either node will be routed to the correct agent actor globally.

## Testing

### Unit and Integration Tests
Run the full test suite (26+ tests):
```bash
cargo test
```

### Code Coverage
We use `cargo-llvm-cov` for precise source-based coverage.

1. **Install components:**
   ```bash
   rustup component add llvm-tools-preview
   cargo install cargo-llvm-cov
   ```

2. **Run coverage:**
   ```bash
   cargo llvm-cov
   ```

3. **Generate HTML report:**
   ```bash
   cargo llvm-cov --html
   open target/llvm-cov/html/index.html
   ```

### Cluster Verification Script
A dedicated script is provided to verify multi-node routing:
```bash
./verify_cluster.sh
```

## Architecture

For a deep dive into the architecture, roadmap, and design decisions, see:
- [Agent.md](Agent.md): Core concepts and features.
- [Plan.md](Plan.md): Implementation milestones and progress tracking.

## License

MIT OR Apache-2.0
