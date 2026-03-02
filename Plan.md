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

## 7. Streaming Output Support
**Objective:** Enable real-time token streaming for improved UX and long-running agent tasks.

- [x] **Streaming Endpoint:**
    - [x] Add `GET /agent/{user_id}/stream` endpoint in `src/main.rs`.
    - [x] Implement streaming handler using actix-web (simplified from WebSocket due to API complexity).
- [x] **Streaming Agent Turn:**
    - [x] Update `UserAgentActor` in `src/actor.rs` to support streaming responses.
    - [x] ZeroClaw doesn't provide native streaming; implemented chunked response handling.
    - [x] Create `AgentStreamTurn` message type that processes tokens.
- [x] **Client-Side Integration:**
    - [x] Update web UI with streaming toggle button.
    - [ ] Handle reconnection and backpressure on the client.
- [ ] **True WebSocket Support:**
    - [ ] Implement WebSocket upgrade using actix-web-actors (requires API migration).
    - [ ] Ensure WebSocket connections can be routed to remote actors via telepathy.
    - [ ] Implement streaming response channel that works across node boundaries.
- [x] **Testing:**
    - [x] Verify streaming endpoint returns chunked responses.
    - [ ] Test streaming across multiple nodes in cluster mode.

## 8. Coding Agent Tools & Workspace Isolation
**Objective:** Extend the tool system to support code development workflows and ensure secure, per-user workspace isolation with dedicated agent personas (Souls).

- [x] **Workspace Isolation & Initialization:**
    - [x] Update `UserAgentActor::init_agent_async` to set a user-specific workspace directory (e.g., `workspaces/{user_id}`).
    - [x] Ensure the workspace directory is created before the agent is initialized.
    - [x] Update `ConfigureAgent` to allow user-defined parameters (e.g., `temperature`, `system_prompt`).
- [ ] **Agent Soul Management:**
    - [ ] Implement `Soul.md` (persona/personality) management in `UserAgentActor`.
    - [ ] Create a default `Soul.md` for new agents in `memory/{user_id}/Soul.md`.
    - [ ] Load `Soul.md` content and incorporate it into the agent's system prompt during initialization.
    - [ ] Allow agents to update their own `Soul.md` (self-reflection/evolution).
- [ ] **Workspace-Aware Coding Tools:**
    - [ ] Refactor `FileReadTool`, `FileWriteTool`, `FileListTool`, `GitTool`, `TerminalTool`, and `CodeRunTool` to be workspace-aware.
    - [ ] Ensure all tool operations are strictly confined to the agent's dedicated `workspaces/{user_id}` directory.
    - [ ] Implement robust path validation to prevent directory traversal outside the workspace.
- [ ] **Built-in ZeroClaw Tools Integration:**
    - [ ] Integrate `FileSearchTool` for efficient workspace searching.
    - [ ] Integrate `TerminalTool` (or `BashTool`) for command execution.
    - [ ] Integrate `BrowserTool` for web research capabilities.
- [ ] **Custom Tool Architecture:**
    - [ ] Create `src/coding_tools.rs` module for all coding-related tools.
    - [ ] Implement base `CodingTool` trait extending ZeroClaw's tool interface.
    - [ ] Add tool registration per-user to allow workspace-specific tools.
- [ ] **File Operations:**
    - [ ] Implement `FileReadTool`: read file contents with path validation (prevent directory traversal).
    - [ ] Implement `FileWriteTool`: create/update files with atomic writes and backup.
    - [ ] Implement `FileListTool`: list directory contents with filtering.
- [ ] **Shell Execution:**
    - [ ] Add command allowlist/denylist configuration for security.
    - [ ] Implement stdout/stderr capture and streaming.
- [ ] **Git Integration:**
    - [ ] Implement `GitTool`: support for `status`, `diff`, `log`, `commit`, `branch` operations.
    - [ ] Add safe-mode configuration to restrict dangerous operations (force push, etc.).
- [ ] **Code Execution Sandbox:**
    - [ ] Research and integrate execution sandbox (Docker, firecracker, or WASM).
    - [ ] Implement `CodeRunTool`: execute code in isolated environment.
    - [ ] Support multiple languages: Python, Node.js, Rust (via `cargo-script` or similar).
    - [ ] Add resource limits (CPU, memory, execution time).
- [x] **Workspace Management:**
    - [x] Implement `WorkspaceTool`: create/clone/delete isolated workspaces per user (Identified as a gap in Findings).
    - [ ] Add workspace templates for common project types.
    - [ ] Implement workspace state persistence across agent restarts.
- [ ] **Testing:**
    - [ ] Test each tool individually with edge cases.
    - [ ] Test tool composition (e.g., read file → modify → write → run tests).
    - [ ] Verify sandbox isolation and resource limits.
    - [ ] Test tool execution across cluster nodes.

## 9. Multi-Agent Team Orchestration
**Objective:** Enable coordinated teams of specialized agents to collaboratively work on coding tasks.

- [ ] **Team Architecture:**
    - [ ] Create `src/team/` module for team orchestration.
    - [ ] Define `TeamSupervisorActor`: coordinates team activities and task distribution.
    - [ ] Implement `TeamConfig`: define team size, roles, and capabilities per team.
    - [ ] Create team templates: Full-Stack, Frontend, Backend, DevOps, etc.
- [ ] **Agent Roles:**
    - [ ] Define role system: Planner, Developer, CodeReviewer, QAEngineer, ExpTester, Documenter.
    - [ ] Implement `PlannerAgent`: decomposes tasks into subtasks with dependencies.
    - [ ] Implement `DeveloperAgent`: writes/modifies code using coding tools.
    - [ ] Implement `QAEngineerAgent`: writes tests, runs test suites.
    - [ ] Implement `CodeReviewerAgent`: reviews changes, suggests improvements.
    - [ ] Implement `ExpTesterAgent`: runs user acceptance tests, validates UX.
- [ ] **Task Management:**
    - [ ] Implement `Task`: atomic unit of work with id, description, status, assignee.
    - [ ] Implement `TaskQueue`: priority queue with dependency tracking.
    - [ ] Implement task assignment logic (round-robin, skill-based, load-balanced).
    - [ ] Add task lifecycle: pending → in-progress → review → done/blocked.
- [ ] **Shared Workspace/Blackboard:**
    - [ ] Implement shared state storage accessible by all team agents.
    - [ ] Add artifact sharing: code patches, test results, docs between agents.
    - [ ] Implement conflict resolution for concurrent edits (last-write-wins or merge).
    - [ ] Add project context: current branch, open PRs, issue tracking.
- [ ] **Workflow Engine:**
    - [ ] Define workflow DSL: `Planning → Dev → QA → Review → ExpTest → Done`.
    - [ ] Support sequential and parallel task execution.
    - [ ] Implement retry logic with exponential backoff on failures.
    - [ ] Add workflow hooks: pre-task, post-task, on-failure callbacks.
- [ ] **Coordination Protocol:**
    - [ ] Implement hub-and-spoke model: TeamSupervisor as coordinator.
    - [ ] Add agent-to-agent messaging for handoffs and collaboration.
    - [ ] Implement decision-making protocol for disagreements (vote, escalate to human).
    - [ ] Add leader election for team continuity during node failures.
- [ ] **Human-in-the-Loop:**
    - [ ] Implement approval gates: require human sign-off at defined stages.
    - [ ] Add user feedback integration: pause/wait for input during workflow.
    - [ ] Create intervention API: allow humans to modify task assignments.
    - [ ] Implement escalation queue for blocked tasks requiring human attention.
- [ ] **Team Observability:**
    - [ ] Implement team-level dashboard: progress, agent status, task timeline.
    - [ ] Add cross-agent tracing: track task flow through multiple agents.
    - [ ] Implement completion tracking: estimated vs actual time per task.
    - [ ] Add team metrics: throughput, block rate, human intervention frequency.
- [ ] **Testing:**
    - [ ] Test team formation and role assignment.
    - [ ] Test task decomposition and dependency resolution.
    - [ ] Test end-to-end workflow: user request → completed code change.
    - [ ] Test human-in-the-loop pauses and approvals.
    - [ ] Test team resilience: simulate node failures during task execution.
