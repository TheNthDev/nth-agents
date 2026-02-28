# Implementation Plan: Fulfilling Project Claims

This plan outlines the steps required to transition the project from its current "walking skeleton" state to a fully functional, distributed AI agent system as described in `Agent.md`.

## 1. ZeroClaw & AI Integration
**Objective:** Replace mocked responses with actual LLM-powered agent turns.

- [x] **Provider Configuration:**
    - [x] Define a `ProviderConfig` struct to handle API keys and model selection (using ZeroClaw defaults/environment).
    - [x] Integrate environment variable loading for sensitive credentials.
    - [x] Support `AGENT_PROVIDER` and `AGENT_MODEL` for dynamic selection.
- [x] **Agent Initialization:**
    - [x] Implement `UserAgentActor::init_agent` in `src/actor.rs` using `zeroclaw::AgentBuilder`.
    - [x] Configure the agent with a default system prompt and selected provider (OpenAI/Synthetic).
    - [x] Register custom tools (e.g., `WeatherTool`) during initialization.
- [x] **Real Turn Execution:**
    - [x] Update `Handler<AgentTurn>` to call `self.agent.turn(msg.message)`.
    - [x] Implement error handling for API failures and rate limits.

## 2. Distributed Cluster Sharding
**Objective:** Enable true cross-node actor communication and ensure user-actor uniqueness.

- [x] **Telepathy Integration:**
    - [x] Implement `impl RemoteActor for UserAgentActor {}` in `src/actor.rs`.
    - [x] Derive `RemoteMessage` for all messages that need to cross node boundaries.
- [x] **Global Actor Registry:**
    - [x] Replace the local `Mutex<HashMap<String, Addr<UserAgentActor>>>` in `src/main.rs` with a cluster-aware registry using `AddrResolver`.
    - [x] Use `actix-telepathy`'s `AddrResolver` to locate actors on other nodes.
- [x] **Remote Routing Logic:**
    - [x] Update `agent_turn` handler to first query the cluster for an existing actor before creating a new one locally.
    - [x] Ensure that if an actor is created, its address is registered in the cluster.

## 3. Scaling & Reliability
**Objective:** Support multiple nodes and maintain state.

- [x] **Node Discovery:**
    - [x] Enhance `src/main.rs` to accept seed node addresses via CLI arguments.
    - [x] Test node joins/leaves and verify heartbeat functionality.
- [x] **Event Logging & Persistence:**
    - [x] Implement persistent storage for agent history/state (using ZeroClaw's local memory backend per user).
    - [x] Ensure that when an actor is recreated, it can reload its state from the `memory/{user_id}`.

## 4. Verification & Testing
- [x] **Unit Tests (TDD):**
    - [x] Implement `test_agent_turn_processing` to verify AI response handling.
    - [x] Implement `test_agent_initialization` to verify ZeroClaw setup.
    - [x] Implement `test_multi_turn_context` to verify agent turn lifecycle.
    - [x] Implement `test_agent_history_persistence` to verify ZeroClaw memory setup.
- [x] **Integration Tests:**
    - [x] Implement `test_agent_turn_endpoint` in `src/main.rs` to verify REST API flow.
    - [x] Implement `test_actor_uniqueness` in `src/main.rs` to verify local sharding/uniqueness.
- [x] **Multi-Node Test Script:** Create a bash script (`verify_cluster.sh`) to launch local instances and verify message routing.
- [x] **LLM Integration Test:** Verify that the agent correctly maintains context over multiple turns and can use registered tools.

## 5. Code Quality & Coverage
**Objective:** Ensure high code quality and maintainability.

- [x] **Code Coverage Integration:** 
    - [x] Install `cargo-llvm-cov` to generate source-based coverage reports.
    - [x] Set a coverage target (e.g., 80%+) for core logic in `src/actor.rs` and `src/main.rs`.
    - [x] Achieve >80% code coverage (Current: ~80% overall, with actor.rs at 80.65%).
    - [x] Integrate coverage reporting into the CI/CD pipeline or local verification script.

    ## 6. User Interface
    **Objective:** Provide a friendly web UI for users to sign up and chat with their agents.

    - [x] **Static Asset Hosting:** Integrate `actix-files` to serve HTML/JS/CSS from a `static` directory.
    - [x] **Web Dashboard:** Implement a responsive single-page application (`index.html`) for agent interaction.
    - [x] **Signup/Session Management:** Create a simple user-id based session entry in the UI.
    - [x] **API Integration:** Connect the UI to the existing `POST /agent/{user_id}/turn` endpoint.
