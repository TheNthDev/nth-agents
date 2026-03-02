### Findings: ZeroClaw Integration and Workspace Management

#### 1. Workspace Definition
In the current implementation of `UserAgentActor` (`src/actor.rs`), the `zeroclaw::agent::Agent` is initialized using its builder, but the `workspace_dir` is not explicitly set.

```rust
// src/actor.rs:208
let agent = Agent::builder()
    .provider(provider)
    .model_name(model_name)
    .tools(tools)
    .memory(memory.clone())
    .observer(observer)
    .tool_dispatcher(Box::new(zeroclaw::agent::dispatcher::NativeToolDispatcher))
    .auto_save(true)
    .build()
    .context("Failed to build zeroclaw agent")?;
```

By default, the ZeroClaw `AgentBuilder` initializes `workspace_dir` to the current directory (`"."`). This means all agents, regardless of the user, share the same workspace directory for any tool operations that rely on it.

While the **memory** component is correctly isolated per user (using `memory/{user_id}/{config_hash}`), the **agent's workspace** itself is not.

#### 2. Utilization of ZeroClaw Capabilities
The project is currently underutilizing ZeroClaw's extensive capabilities:

*   **Limited Tooling:** The only custom tool implemented is `WeatherTool`. ZeroClaw provides a suite of powerful built-in tools (e.g., `FileSearchTool`, `TerminalTool`, `BrowserTool`) that are not being leveraged.
*   **Workspace Isolation:** As mentioned, the lack of a per-user `workspace_dir` means the agent cannot safely perform file operations or run terminal commands without risk of data leakage or cross-user interference.
*   **Prompt Customization:** The project uses the default `SystemPromptBuilder`, missing out on tailored system instructions that could better define the agent's persona and available capabilities.
*   **Incomplete Roadmap:** The `Plan.md` lists several workspace-related features (e.g., `WorkspaceTool`, isolated workspaces per user) that are still marked as "not yet started."

#### 3. Recommendations
To better utilize ZeroClaw and improve security:
1.  **Isolate Workspaces:** Update `UserAgentActor::init_agent_async` to set a user-specific workspace directory using `.workspace_dir(PathBuf::from(format!("workspaces/{}", user_id)))`.
2.  **Expose More Tools:** Integrate more of ZeroClaw's built-in tools to increase the agents' utility.
3.  **Refine Config:** Allow users to configure more agent parameters (like `temperature`, `system_prompt`) via the `ConfigureAgent` message.
