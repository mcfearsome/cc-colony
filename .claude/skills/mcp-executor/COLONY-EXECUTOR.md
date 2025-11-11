# Colony MCP Executor Skill

This is a colony-aware adaptation of the Code Executor Skill for CC-Colony. It enables a dedicated MCP executor agent that processes MCP tool requests from other agents via the colony messaging system.

## Overview

The Colony MCP Executor runs as a long-running agent in a dedicated pane that:
1. Watches the executor message queue for incoming tasks
2. Executes multi-tool MCP workflows using TypeScript or Python
3. Returns results via colony messages
4. Provides transparent progress updates

## Architecture

```
┌─────────────┐         ┌──────────────────┐         ┌─────────────┐
│   Agent A   │         │  MCP Executor    │         │ MCP Servers │
│             │ ──msg──>│  (Long-running)  │<─stdio─>│ (Various)   │
│  Requests   │         │                  │         │             │
│  MCP Task   │ <─msg───│  Returns Result  │         │             │
└─────────────┘         └──────────────────┘         └─────────────┘
```

## For Executor Agent: Processing Tasks

When you are the executor agent, you will:

### 1. Monitor the Task Queue

Check for new executor tasks regularly:

```bash
# Read executor tasks (these are in your messages inbox)
./colony_message.sh read
```

### 2. Parse Task Requests

Executor task messages will contain descriptions of MCP workflows to execute:

```
Execute MCP workflow: Export user data to CSV

Pattern: multi-tool-workflow
Language: typescript
Tools needed:
- mcp__database__query (SELECT * FROM users)
- mcp__filesystem__writeFile (write to /tmp/users.csv)

Expected output: CSV file with all user records
```

### 3. Execute the MCP Workflow Directly

**You have MCP servers configured and ready to use.** Execute the workflow directly:

**Option A: Use Bash to run TypeScript (Deno)**
```bash
# Create the execution script
cat > /tmp/mcp_task.ts <<'EOF'
import { callMCPTool } from "./.claude/skills/mcp-executor/lib/mcp-client.ts";

// Step 1: Query database
const users = await callMCPTool("mcp__database__query", {
  query: "SELECT * FROM users"
});

// Step 2: Transform to CSV
const csv = users.map(u => `${u.id},${u.name},${u.email}`).join('\n');

// Step 3: Write file
await callMCPTool("mcp__filesystem__writeFile", {
  path: "/tmp/users.csv",
  content: csv
});

console.log(JSON.stringify({ success: true, records: users.length }));
EOF

# Execute with Deno
deno run --allow-read --allow-run --allow-env /tmp/mcp_task.ts
```

**Option B: Use Bash to run Python**
```bash
# Create the execution script
cat > /tmp/mcp_task.py <<'EOF'
import sys
sys.path.insert(0, './.claude/skills/mcp-executor')
from lib.mcp_client import call_mcp_tool
import json

# Step 1: Query database
users = await call_mcp_tool("mcp__database__query", {
    "query": "SELECT * FROM users"
})

# Step 2: Transform to CSV
csv = '\n'.join([f"{u['id']},{u['name']},{u['email']}" for u in users])

# Step 3: Write file
await call_mcp_tool("mcp__filesystem__writeFile", {
    "path": "/tmp/users.csv",
    "content": csv
})

print(json.dumps({"success": True, "records": len(users)}))
EOF

# Execute with Python
python3 /tmp/mcp_task.py
```

**Option C: Reference existing script patterns**
```bash
# Copy and adapt an existing pattern
cp ./.claude/skills/mcp-executor/scripts/typescript/multi-tool-workflow.ts /tmp/task.ts
# Edit the file to match the requested workflow
# Then execute it
deno run --allow-read --allow-run --allow-env /tmp/task.ts
```

### 4. Send Result Back

Reply to the requesting agent with the result:

```bash
# Send success result
./colony_message.sh send backend-1 "MCP task completed: [SUMMARY]

Result:
{json_result_here}

Status: SUCCESS"

# Or send error
./colony_message.sh send backend-1 "MCP task failed: [ERROR_DETAILS]

Status: ERROR"
```

### 5. Best Practices for Executor Agent

- **Acknowledge immediately**: Send a quick "Task received, executing..." message
- **Progress updates**: For long-running tasks, send periodic status updates
- **Error details**: Include full error messages and context for failures
- **Resource management**: Track concurrent executions and queue management
- **Logging**: Keep detailed logs of all executions for debugging

## For Other Agents: Requesting MCP Tasks

When you need to execute complex MCP workflows, send a task to the executor:

### 1. Check if Executor is Available

```bash
./colony_message.sh list-agents
```

Look for an agent with ID like `mcp-executor` or role containing "executor".

### 2. Format Your Request

Create a task message with the MCP workflow details:

```bash
./colony_message.sh send mcp-executor "Execute MCP workflow: Fetch user data from database and generate report

Pattern: multi-tool-workflow
Language: typescript
Tools needed:
- mcp__database__query
- mcp__filesystem__writeFile

Description: Query users table, transform results, write JSON report to /tmp/report.json"
```

### 3. Wait for Response

The executor will send back:
- **Acknowledgment**: "Task received, executing..."
- **Progress updates**: "Step 1/3 completed..."
- **Final result**: "MCP task completed: ..." with the result data

### 4. When to Use the Executor

**Good use cases:**
- Complex multi-tool MCP workflows (3+ tool calls)
- Operations requiring specialized MCP servers
- Parallel MCP tool execution
- Retry logic and error recovery patterns
- Batch file processing via MCP

**Bad use cases:**
- Single simple tool calls (use direct MCP servers instead)
- Tasks not involving MCP tools
- Real-time interactive operations

## Executor Configuration

The executor agent is configured in `colony.yml`:

```yaml
executor:
  enabled: true
  agent_id: mcp-executor
  mcp_config_path: ~/.claude/subagent-mcp.json
  languages:
    - typescript  # Requires Deno
    - python      # Requires Python 3.8+
```

## MCP Configuration File

The executor uses a separate MCP config file (`~/.claude/subagent-mcp.json`) to avoid loading all MCP schemas into every agent:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "database": {
      "command": "uvx",
      "args": ["mcp-server-sqlite", "--db-path", "/path/to/db.sqlite"]
    },
    "git": {
      "command": "uvx",
      "args": ["mcp-server-git", "--repository", "/path/to/repo"]
    }
  }
}
```

## Available Script Patterns

Reference these patterns when requesting tasks:

### TypeScript (Deno)
- `multi-tool-workflow` - Sequential data pipeline
- `parallel-execution` - Concurrent tool calls
- `error-recovery` - Retry logic with fallbacks

### Python
- `multi_tool_workflow` - Sequential data pipeline
- `parallel_execution` - Concurrent tool calls
- `error_recovery` - Retry logic with fallbacks

## Monitoring via TUI

The colony TUI includes an "Executor" tab showing:
- Queue depth (pending tasks)
- Currently executing task
- Recent completions with status
- Error rate and performance metrics

Press `5` to view the Executor tab.

## Example: Complete Workflow

### Agent requests database export:

```bash
./colony_message.sh send mcp-executor "Execute MCP workflow: Export all user records to CSV

Pattern: multi-tool-workflow
Language: typescript
Tools:
- mcp__database__query (SELECT * FROM users)
- mcp__filesystem__writeFile (write to /tmp/users.csv)

Expected output: CSV file with all user records"
```

### Executor processes:

1. Receives message
2. Sends acknowledgment
3. Launches Task tool with TypeScript subagent
4. Subagent writes code using multi-tool-workflow pattern
5. Executes: database query → transform to CSV → write file
6. Returns result to requesting agent

### Requesting agent receives:

```
MCP task completed: Exported 1,247 user records to CSV

Result:
{
  "success": true,
  "records_exported": 1247,
  "file_path": "/tmp/users.csv",
  "file_size": "245KB",
  "duration": "3.2s"
}

Status: SUCCESS
```

## Token Efficiency

By using the executor pattern:
- **Main agents**: Stay lightweight without MCP schemas (98% token reduction)
- **Executor agent**: Loads MCP schemas only when needed via subagents
- **Subagents**: Execute code and terminate, freeing resources
- **Overall**: Massive token savings across the colony

## Troubleshooting

### Executor not responding
- Check if executor pane is running: `tmux ls`
- View executor logs: `cat .colony/logs/mcp-executor.log`
- Verify MCP config exists: `ls -la ~/.claude/subagent-mcp.json`

### Task execution failures
- Check MCP server configurations
- Verify required tools are available
- Review subagent error messages
- Ensure proper permissions for Deno/Python

### Performance issues
- Reduce concurrent executions
- Use parallel patterns for independent operations
- Optimize MCP tool call sequences
- Monitor resource usage in TUI

## See Also

- **Base skill**: `.claude/skills/mcp-executor/SKILL.md` - Original executor documentation
- **Colony messaging**: `.claude/skills/colony-message.md` - Message system guide
- **Script examples**: `.claude/skills/mcp-executor/scripts/` - Reference implementations
- **Templates**: `.claude/skills/mcp-executor/templates/` - Starting points for custom workflows
