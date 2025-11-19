# Manifest.build UI Integration Validation

**Date**: 2025-11-19
**Purpose**: Validate the feasibility of using manifest.build-style adhoc UIs for enhanced user interactions in cc-colony

---

## Executive Summary

This document validates the idea of integrating web-based adhoc UIs (using manifest.build or similar frameworks) into cc-colony for:
- **Complex input prompts** (multi-field forms, conditional logic)
- **Rich data visualization** (graphs, charts, complex tables)
- **Live widgets** (real-time metrics, interactive dashboards)
- **Dynamic interfaces** (agent-generated UIs based on task context)

**Verdict**: ✅ **HIGHLY VIABLE** - The architecture is well-suited for this, with minimal integration effort.

---

## 1. Current State Analysis

### 1.1 Existing UI Mechanisms

cc-colony currently uses three interaction paradigms:

| Mechanism | Use Case | Files | Limitations |
|-----------|----------|-------|-------------|
| **Ratatui TUI** | Real-time monitoring, navigation | `src/colony/tui/*.rs` | Text-only, no rich graphics |
| **Dialoguer CLI** | Simple prompts (text, select, confirm) | `src/utils.rs:57-114` | Limited to basic input types |
| **Web (OAuth)** | Authentication flows | `src/colony/auth/oauth.rs` | Single-purpose, not extensible |

### 1.2 Pain Points

1. **Graph/Chart Visualization**: Cannot display task dependencies as visual graphs
2. **Complex Forms**: Multi-step dialogs (CreateTask: 5 steps) are clunky in CLI
3. **Real-time Updates**: TUI refreshes every 2s, but can't show live streaming data elegantly
4. **Agent-Driven UIs**: Agents can't dynamically create custom UIs for specific tasks
5. **Data Exploration**: Large datasets (logs, metrics) are hard to explore in terminal

---

## 2. What is Manifest.build?

### 2.1 Core Concept

Manifest is a **YAML-defined backend + auto-generated admin UI** framework:

```yaml
# manifest.yml - Example
entities:
  - name: Task
    properties:
      - name: title
        type: string
      - name: priority
        type: number
        min: 1
        max: 5
      - name: status
        type: enum
        values: [pending, in-progress, completed]
```

**Generates**:
- REST API (`/api/tasks`)
- Admin panel UI at `http://localhost:1111` with CRUD operations
- Built-in auth and validation

### 2.2 Key Features for Our Use Case

| Feature | Benefit for cc-colony |
|---------|----------------------|
| **Instant CRUD UIs** | Could generate task/agent management UIs from colony data models |
| **Single File Backend** | Minimal overhead - drop in `manifest.yml` and start |
| **Auto REST API** | Colony agents could interact via HTTP instead of disk files |
| **Built-in Auth** | Could integrate with existing OAuth system |
| **Hot Reload** | Modify UI structure without restarting |

---

## 3. Integration Architecture

### 3.1 Proposed Design

```
┌─────────────────────────────────────────────────────────────┐
│                        Colony Core                           │
│  (Rust - Current orchestration, task management, messaging)  │
└───────────────┬─────────────────────────────────────────────┘
                │
                ├─────► TUI (ratatui) - Monitoring & Navigation
                │
                ├─────► CLI (dialoguer) - Quick prompts
                │
                └─────► Web UI Pane (NEW)
                         │
                         ├─► Manifest.build backend (Node.js)
                         │   - YAML-defined entities
                         │   - Auto REST API (:1111/api)
                         │   - Admin panel (:1111)
                         │
                         └─► Custom Web Views (Optional)
                             - React/Vue for complex widgets
                             - D3.js for graph visualization
                             - WebSocket for live updates
```

### 3.2 Integration Points

#### Option A: Dedicated Pane (Recommended)
- Add new TUI tab: `[Agents] [Tasks] [Messages] [State] [Help] [Web UI]`
- Launches browser to `http://localhost:1111` when accessed
- Backend starts as subprocess when colony starts

#### Option B: Embedded Browser
- Use `tauri` or `webview` crate to embed browser in TUI
- Inline rendering (more complex but seamless)

#### Option C: Side-by-Side Tmux
- New tmux pane alongside agent panes
- Launches `w3m` or `lynx` for terminal-based browsing

### 3.3 Data Flow

```
Colony Agent Request
       ↓
Colony Core (Rust)
       ↓
HTTP POST to Manifest backend
  (http://localhost:1111/api/prompt)
       ↓
Manifest serves UI form
       ↓
User fills form in browser
       ↓
Manifest POST to Colony webhook
       ↓
Colony writes response to agent's input buffer
       ↓
Agent receives structured response
```

---

## 4. Use Case Validation

### 4.1 Complex Input Prompts

**Current (5-step CLI dialog)**:
```
src/colony/tui/app.rs:233-278

DialogStep::CreateTaskId → CreateTaskTitle → CreateTaskDescription →
CreateTaskAssignedTo → CreateTaskPriority
```

**With Manifest**:
```yaml
entities:
  - name: TaskPrompt
    properties:
      - name: id
        type: string
        required: true
      - name: title
        type: string
        required: true
      - name: description
        type: text
      - name: assigned_to
        type: relation
        target: Agent
      - name: priority
        type: enum
        values: [low, medium, high, critical]
```

**Result**: Single-page form with validation, dropdown for agents, all in one view.

### 4.2 Graph Visualization

**Current**: Cannot visualize task dependencies
**With Web UI**: Use D3.js/Cytoscape.js to render dependency graphs

```javascript
// Example: Visualize agent task graph
fetch('http://localhost:1111/api/tasks')
  .then(tasks => renderGraph(tasks))
```

### 4.3 Live Widgets

**Current**: TUI polls every 2 seconds (`src/colony/tui/app.rs:349`)
**With WebSocket**:

```javascript
const ws = new WebSocket('ws://localhost:1111/ws');
ws.onmessage = (event) => {
  updateDashboard(JSON.parse(event.data));
};
```

**Enables**: Real-time agent logs, live CPU/memory graphs, streaming task updates

### 4.4 Agent-Driven UIs

**Concept**: Agents could request custom UIs via messages

```bash
# Agent sends message
colony message broadcast "I need user input for deployment config"

# Colony receives, launches UI prompt
colony ui prompt --type deployment-config --agent backend-1
```

---

## 5. Technical Feasibility

### 5.1 Existing Infrastructure ✅

Already have components for this:

| Component | Location | Ready? |
|-----------|----------|--------|
| **Local HTTP Server** | `src/colony/auth/oauth.rs:24-129` | ✅ Yes (port 8888) |
| **Browser Launch** | Uses `open` crate | ✅ Yes |
| **JSON Serialization** | `ColonyData` has `#[derive(Serialize)]` | ✅ Yes |
| **REST Endpoints** | None yet | ❌ Need to add |
| **Subprocess Management** | Tmux pane creation | ✅ Yes |

### 5.2 Dependencies to Add

```toml
# Cargo.toml additions
[dependencies]
axum = "0.7"           # Web framework (Rust-native alternative to manifest)
tower-http = "0.5"     # CORS, logging
tokio-tungstenite = "0.21"  # WebSocket support
```

**Alternative**: Use manifest.build directly (Node.js) as sidecar process

### 5.3 Implementation Estimate

| Task | Effort | Notes |
|------|--------|-------|
| Basic HTTP server (Axum) | 2-3 hours | Similar to OAuth server |
| REST API for colony data | 4-5 hours | Expose tasks, agents, messages |
| Manifest integration | 2-3 hours | Write YAML schema, start subprocess |
| TUI "Web UI" tab | 2-3 hours | Add tab, browser launch |
| WebSocket live updates | 4-6 hours | Stream agent logs, metrics |
| Custom UI templates | Variable | Depends on complexity |

**Total**: ~15-20 hours for full integration

---

## 6. Pros & Cons

### 6.1 Advantages

✅ **Richer Interactions**: Forms, graphs, charts not possible in TUI
✅ **Faster Prototyping**: Manifest auto-generates UIs from YAML
✅ **Better for Complex Data**: Tables with sorting/filtering/pagination
✅ **Live Updates**: WebSocket beats 2s polling
✅ **Agent Autonomy**: Agents could trigger UIs programmatically
✅ **Familiar UX**: Many users prefer web UIs over TUIs
✅ **Extensible**: Easy to add custom React/Vue components

### 6.2 Disadvantages

❌ **Extra Dependency**: Adds Node.js if using manifest.build
❌ **Network Overhead**: HTTP/WebSocket vs direct disk I/O
❌ **Port Management**: Need to avoid conflicts (currently using 8888)
❌ **Security**: Need CSRF protection, auth tokens
❌ **Browser Requirement**: Not all environments have browsers
❌ **Complexity**: One more system to debug

---

## 7. Alternative Approaches

### 7.1 Pure Rust Solution (Recommended)

Instead of manifest.build, use Rust ecosystem:

```rust
// src/colony/web_ui/server.rs
use axum::{Router, routing::get};
use tower_http::services::ServeDir;

async fn serve_web_ui() {
    let app = Router::new()
        .route("/api/tasks", get(list_tasks))
        .nest_service("/", ServeDir::new("web_ui/dist"));

    axum::Server::bind(&"0.0.0.0:1111".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**Benefits**:
- No Node.js dependency
- Full control over implementation
- Better performance
- Type safety with Rust

### 7.2 Hybrid: Use Manifest for Prototyping

1. Start with manifest.build for rapid iteration
2. Identify most-used UIs
3. Reimplement critical ones in Rust
4. Keep manifest for experimental/adhoc UIs

### 7.3 Terminal-Native Improvements

Could improve TUI instead:

- Use `ratatui-image` for inline graphs (limited)
- Better multi-step forms with validation
- Sparklines for metrics (ASCII charts)

**Verdict**: Doesn't solve rich visualization problem

---

## 8. Concrete Implementation Proposal

### 8.1 Phase 1: Foundation (Week 1)

1. **Add Axum web server** to colony
   - Serve static files from `web_ui/` directory
   - Basic REST API: `/api/colony/status`, `/api/agents`, `/api/tasks`
   - Start on port 1111 alongside colony start

2. **TUI Integration**
   - Add "Web UI" tab (or press 'W' key)
   - Opens browser to `http://localhost:1111`

3. **Basic Template**
   - Simple HTML dashboard showing agent status
   - No manifest yet - just prove the concept

### 8.2 Phase 2: Manifest Integration (Week 2)

1. **Install manifest.build as dev dependency**
   ```bash
   npm install -D @mnfst/manifest
   ```

2. **Create `manifest.yml`** schema
   ```yaml
   entities:
     - name: Task
       properties:
         - {name: id, type: string}
         - {name: title, type: string}
         - {name: status, type: enum, values: [pending, in_progress, blocked, completed]}
   ```

3. **Start manifest as subprocess**
   ```rust
   Command::new("npx")
       .args(["manifest", "dev"])
       .spawn()?;
   ```

4. **Sync colony data → manifest**
   - POST tasks/agents to manifest's REST API
   - Manifest becomes UI layer over colony data

### 8.3 Phase 3: Advanced Features (Week 3)

1. **WebSocket live updates**
   - Stream agent logs to browser
   - Real-time task status changes

2. **Custom widgets**
   - Task dependency graph (D3.js)
   - Agent CPU/memory charts
   - Message timeline view

3. **Agent-triggered UIs**
   - Agent sends special message format
   - Colony parses and launches UI prompt
   - Response sent back to agent

### 8.4 Phase 4: Production Hardening (Week 4)

1. **Security**
   - Add auth tokens (reuse OAuth system)
   - CSRF protection
   - Rate limiting

2. **Error handling**
   - Graceful fallback if web UI fails
   - Port conflict detection

3. **Documentation**
   - User guide for web UI features
   - Developer guide for adding custom UIs

---

## 9. Example Use Cases

### 9.1 Task Dependency Graph

**Before**:
```bash
colony tasks list
# Text output, hard to see dependencies
```

**After**:
- Open Web UI tab
- See interactive graph with nodes (tasks) and edges (dependencies)
- Click node to see details
- Drag to reorder priorities

### 9.2 Agent Performance Dashboard

**Before**: TUI shows basic stats
**After**: Live charts with:
- CPU usage over time (line chart)
- Task completion rates (bar chart)
- Message throughput (area chart)
- All updating in real-time via WebSocket

### 9.3 Complex Deployment Workflow

**Agent Scenario**: Backend agent needs deployment config

**Before**:
```
Agent: "I need database URL, API keys, feature flags..."
[User types 5 separate CLI prompts]
```

**After**:
```
Agent: colony ui prompt --template deployment
[Browser opens with pre-filled form]
[User fills all fields at once]
[Agent receives JSON: {db_url: "...", api_key: "...", ...}]
```

---

## 10. Recommendation

### ✅ **Proceed with Implementation**

**Recommended Approach**: **Hybrid (Rust + Optional Manifest)**

1. **Start with Pure Rust** (Axum web server)
   - Proves concept without extra dependencies
   - Full control over implementation
   - Better performance

2. **Add Manifest.build as Optional Feature**
   ```bash
   colony start --enable-manifest
   ```
   - Useful for rapid prototyping of new UIs
   - Teams with Node.js familiarity can use it
   - Not required for core functionality

3. **Prioritize High-Impact Features**
   - Task dependency visualization (high value)
   - Live agent logs streaming (high value)
   - Complex forms (medium value)
   - Custom agent UIs (low value initially, high long-term)

### 10.1 Success Metrics

After implementation, we should see:
- ✅ Reduced time to create complex tasks (5 steps → 1 form)
- ✅ Better visibility into agent relationships (visual graphs)
- ✅ Faster debugging (live log streaming)
- ✅ Increased user satisfaction (web UI preference survey)

### 10.2 Risk Mitigation

- **Port conflicts**: Auto-detect and increment port if 1111 is taken
- **No browser**: Fall back to CLI mode, detect headless environments
- **Node.js missing**: Make manifest optional, detect at runtime
- **Security**: Start with localhost-only, add auth later

---

## 11. Next Steps

1. **Validate with stakeholders** - Get feedback on this proposal
2. **Create POC** - Build Phase 1 (basic web server + REST API)
3. **User testing** - Show to 2-3 users, gather feedback
4. **Iterate** - Refine based on feedback
5. **Document** - Write user guide and API docs
6. **Launch** - Merge to main, announce feature

---

## Appendix A: Code Locations for Integration

| Component | File | Lines | Notes |
|-----------|------|-------|-------|
| Colony startup | `src/colony/start.rs` | 1-300 | Add web server startup here |
| TUI tabs | `src/colony/tui/app.rs` | 30-52 | Add WebUI tab to `Tab` enum |
| OAuth server | `src/colony/auth/oauth.rs` | 24-129 | Reference for HTTP server pattern |
| Data models | `src/types.rs` | All | Already has `#[derive(Serialize)]` |
| Message handling | `src/colony/messages_cmd.rs` | All | Hook for agent UI requests |

---

## Appendix B: Manifest.build Alternatives

| Tool | Language | Pros | Cons |
|------|----------|------|------|
| **Manifest.build** | Node.js | Auto-gen UIs, YAML config | Node dependency |
| **Axum + Askama** | Rust | Native, fast, type-safe | More boilerplate |
| **Tauri** | Rust + JS | Desktop app, webview | Heavier dependency |
| **Retool** | SaaS | Rapid development | Cloud-only, cost |
| **Grafana** | Go | Amazing dashboards | Overkill for forms |

**Verdict**: Axum (Rust) for core, manifest.build for experimentation

---

## Conclusion

Using manifest.build-style adhoc UIs in cc-colony is **not only viable but recommended**. The architecture already has most primitives needed (HTTP server, browser launch, JSON serialization), and the benefits (richer UIs, better visualization, agent autonomy) significantly outweigh the costs (minor complexity, optional dependency).

**Recommendation**: Start with Phase 1 POC using pure Rust (Axum), validate with users, then expand based on feedback.
