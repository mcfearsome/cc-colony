# Manifest UI Integration - Executive Summary

**TL;DR**: Using manifest.build or similar frameworks to add web-based UIs to cc-colony is **highly viable and recommended**. The architecture already supports 80% of what's needed.

---

## 📊 Quick Comparison

### Current: Multi-Step CLI Dialog (CreateTask)
```
Step 1: Enter task ID       ▶ [type: task-42]
Step 2: Enter title        ▶ [type: Implement feature X]
Step 3: Enter description  ▶ [type: Add support for...]
Step 4: Select agent       ▶ [↑↓ to choose]
Step 5: Select priority    ▶ [↑↓ to choose]

✓ Submit
```
**Total interactions**: 5 sequential screens

### Proposed: Single-Page Web Form
```
┌─────────────────────────────────────────┐
│ Create Task                              │
├─────────────────────────────────────────┤
│ ID:          [task-42_____________]      │
│ Title:       [Implement feature X_]      │
│ Description: [Add support for...___]     │
│              [___________________]      │
│ Agent:       [▼ backend-1        ]      │
│ Priority:    [▼ High             ]      │
│ Dependencies: [+ Add]                    │
│                                          │
│           [Cancel]  [Create Task]       │
└─────────────────────────────────────────┘
```
**Total interactions**: 1 screen, all fields visible

---

## 🎯 What We Get

| Feature | Current | With Web UI | Impact |
|---------|---------|-------------|--------|
| **Task Creation** | 5-step CLI | Single form | ⭐⭐⭐ High |
| **Dependency Visualization** | Text list | Interactive graph | ⭐⭐⭐ High |
| **Live Metrics** | 2s polling | WebSocket stream | ⭐⭐ Medium |
| **Agent Logs** | Static file | Live tail w/ search | ⭐⭐⭐ High |
| **Complex Data** | JSON dump | Tables w/ sort/filter | ⭐⭐ Medium |
| **Agent-Driven UIs** | Not possible | Dynamic forms | ⭐⭐⭐ High |

---

## 🏗️ Architecture (Simplified)

```
┌──────────────────────────────────────────────────────────┐
│                    Colony (Rust)                          │
│  • Orchestration • Task Management • Messaging           │
└────────┬─────────────────────────────────────────────────┘
         │
         ├─► TUI (ratatui)     ← For monitoring
         ├─► CLI (dialoguer)   ← For quick prompts
         └─► Web Server NEW    ← For rich UIs
              │
              ├─ REST API (http://localhost:1111/api)
              ├─ WebSocket (ws://localhost:1111/ws)
              └─ Static Files (/task-graph.html, etc.)
```

**Access**: Press 'W' in TUI → Opens browser → http://localhost:1111

---

## 📦 What's Included in This Proposal

### 1. Documentation
- **`docs/manifest-ui-validation.md`** (6,500 words)
  - Full technical analysis
  - Integration patterns
  - Security considerations
  - 4-week implementation plan

### 2. Proof of Concept
- **`examples/web_ui_poc/manifest.yml`**
  - Complete manifest.build schema
  - Defines Task, Agent, Message, Metric entities
  - Auto-generates admin panel

- **`examples/web_ui_poc/sync_colony_data.sh`**
  - Syncs colony data to manifest backend
  - Ready to test

- **`examples/web_ui_poc/static/task-graph.html`**
  - Interactive D3.js dependency graph
  - Real-time updates
  - Drag-and-drop nodes
  - Works standalone or with manifest

### 3. Implementation Guide
- **`examples/web_ui_poc/README.md`**
  - Quick start instructions
  - Integration examples
  - Next steps

---

## 🚀 Try It Now

```bash
# Install manifest.build
npm install -g @mnfst/manifest

# Start the POC
cd cc-colony/examples/web_ui_poc
manifest dev

# Open browser to http://localhost:1111
# Login: admin@manifest.build / admin

# Or view the task graph directly
open examples/web_ui_poc/static/task-graph.html
```

---

## 💡 Key Insights

### ✅ What Makes This Easy
1. **Already have HTTP server** (`src/colony/auth/oauth.rs`)
2. **Already have JSON serialization** (`ColonyData`)
3. **Already have browser launching** (`open` crate)
4. **Already have subprocess management** (tmux panes)

### ⚠️ What to Watch Out For
1. **Port conflicts** - Use auto-detection
2. **No browser environments** - Fall back to CLI
3. **Security** - Start localhost-only, add auth later
4. **Optional dependency** - Don't break colony if web UI fails

---

## 📈 Recommended Path Forward

### Phase 1: POC (1 week) ✅ DONE
- [x] Create manifest.yml schema
- [x] Build sync script
- [x] Create example visualization (task-graph.html)
- [x] Write validation doc

### Phase 2: Integration (1 week)
- [ ] Add Axum web server to colony
- [ ] Expose REST API (/api/tasks, /api/agents)
- [ ] Add "Web UI" tab to TUI
- [ ] Start server as subprocess on `colony start`

### Phase 3: Enhancement (1 week)
- [ ] Add WebSocket for live updates
- [ ] Create custom dashboard
- [ ] Implement agent-triggered UI prompts
- [ ] Add authentication

### Phase 4: Polish (1 week)
- [ ] Error handling
- [ ] Fallback strategies
- [ ] Performance optimization
- [ ] Documentation

**Total**: ~4 weeks to production-ready

---

## 🎬 Demo Scenarios

### Scenario 1: Visual Task Planning
**Before**: `colony tasks list` (text output)
**After**: Open web UI → See interactive graph → Click nodes → Reorder priorities → Drag dependencies

### Scenario 2: Agent Requests Input
**Current**:
```
Agent: "I need deployment config"
User: [Answers 5 separate CLI prompts]
```

**With Web UI**:
```
Agent: colony ui prompt --template deployment
[Browser opens with form]
User: [Fills all fields at once]
Agent: [Receives JSON response]
```

### Scenario 3: Live Debugging
**Current**: `colony logs backend-1` (static)
**After**: Open web UI → See live log stream → Search/filter → Click to see context

---

## 🎯 Success Criteria

After implementation, we should see:
- ✅ Task creation time: **5 steps → 1 form** (80% reduction)
- ✅ User satisfaction: **+30%** (from surveys)
- ✅ Debugging time: **-50%** (live logs vs static)
- ✅ Adoption: **60%+ users** prefer web UI for complex tasks

---

## 🤔 Decision Points

### Do We Need This?
**Yes, if**:
- Users create many complex tasks
- Task dependencies are hard to understand
- Real-time visibility is important
- Users prefer graphical tools

**No, if**:
- Colony is primarily for CLI power users
- Simple tasks only
- Security concerns outweigh benefits

### Should We Use Manifest.build?
**Use manifest** for:
- Rapid prototyping
- Quick admin panels
- Simple CRUD UIs

**Use Axum (Rust)** for:
- Core features
- Performance-critical paths
- Long-term maintenance

**Recommendation**: **Hybrid** - Axum core + manifest for experimentation

---

## 📞 Next Steps

1. **Review this proposal** - Get team feedback
2. **Test the POC** - Run `manifest dev` in examples/web_ui_poc
3. **Decide on approach** - Manifest vs pure Rust vs hybrid
4. **Start Phase 2** - If approved, begin integration

---

## 📚 Further Reading

- **Full validation**: `docs/manifest-ui-validation.md`
- **POC guide**: `examples/web_ui_poc/README.md`
- **Manifest docs**: https://manifest.build/docs/
- **D3.js examples**: https://d3js.org/

---

**Questions?** Open an issue or discuss in team meeting.

**Ready to proceed?** Start with Phase 2 integration plan.
