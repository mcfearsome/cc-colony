/// MCP Executor module for colony-aware MCP task execution
use crate::error::ColonyResult;
use crate::colony::config::ExecutorConfig;
use std::fs;
use std::path::Path;

// Embed skill files at compile time
const SKILL_MD: &str = include_str!("../../.claude/skills/mcp-executor/SKILL.md");
const COLONY_EXECUTOR_MD: &str = include_str!("../../.claude/skills/mcp-executor/COLONY-EXECUTOR.md");
const MCP_CLIENT_TS: &str = include_str!("../../.claude/skills/mcp-executor/lib/mcp-client.ts");
const MCP_CLIENT_PY: &str = include_str!("../../.claude/skills/mcp-executor/lib/mcp_client.py");

// Embed templates
const BASIC_TS_TEMPLATE: &str = include_str!("../../.claude/skills/mcp-executor/templates/basic-typescript.template.ts");
const BASIC_PY_TEMPLATE: &str = include_str!("../../.claude/skills/mcp-executor/templates/basic-python.template.py");
const MULTI_TOOL_TS_TEMPLATE: &str = include_str!("../../.claude/skills/mcp-executor/templates/multi-tool.template.ts");
const MULTI_TOOL_PY_TEMPLATE: &str = include_str!("../../.claude/skills/mcp-executor/templates/multi-tool.template.py");

// Embed TypeScript scripts
const MULTI_TOOL_WORKFLOW_TS: &str = include_str!("../../.claude/skills/mcp-executor/scripts/typescript/multi-tool-workflow.ts");
const PARALLEL_EXECUTION_TS: &str = include_str!("../../.claude/skills/mcp-executor/scripts/typescript/parallel-execution.ts");
const ERROR_RECOVERY_TS: &str = include_str!("../../.claude/skills/mcp-executor/scripts/typescript/error-recovery.ts");

// Embed Python scripts
const MULTI_TOOL_WORKFLOW_PY: &str = include_str!("../../.claude/skills/mcp-executor/scripts/python/multi_tool_workflow.py");
const PARALLEL_EXECUTION_PY: &str = include_str!("../../.claude/skills/mcp-executor/scripts/python/parallel_execution.py");
const ERROR_RECOVERY_PY: &str = include_str!("../../.claude/skills/mcp-executor/scripts/python/error_recovery.py");

/// Extract embedded skill files to the repository .claude/skills directory
fn extract_skill_files(colony_root: &Path) -> ColonyResult<()> {
    let skill_dir = colony_root.join(".claude/skills/mcp-executor");

    // Create directory structure
    fs::create_dir_all(skill_dir.join("lib"))?;
    fs::create_dir_all(skill_dir.join("templates"))?;
    fs::create_dir_all(skill_dir.join("scripts/typescript"))?;
    fs::create_dir_all(skill_dir.join("scripts/python"))?;

    // Write main documentation
    fs::write(skill_dir.join("SKILL.md"), SKILL_MD)?;
    fs::write(skill_dir.join("COLONY-EXECUTOR.md"), COLONY_EXECUTOR_MD)?;

    // Write library files
    fs::write(skill_dir.join("lib/mcp-client.ts"), MCP_CLIENT_TS)?;
    fs::write(skill_dir.join("lib/mcp_client.py"), MCP_CLIENT_PY)?;

    // Write templates
    fs::write(skill_dir.join("templates/basic-typescript.template.ts"), BASIC_TS_TEMPLATE)?;
    fs::write(skill_dir.join("templates/basic-python.template.py"), BASIC_PY_TEMPLATE)?;
    fs::write(skill_dir.join("templates/multi-tool.template.ts"), MULTI_TOOL_TS_TEMPLATE)?;
    fs::write(skill_dir.join("templates/multi-tool.template.py"), MULTI_TOOL_PY_TEMPLATE)?;

    // Write TypeScript scripts
    fs::write(skill_dir.join("scripts/typescript/multi-tool-workflow.ts"), MULTI_TOOL_WORKFLOW_TS)?;
    fs::write(skill_dir.join("scripts/typescript/parallel-execution.ts"), PARALLEL_EXECUTION_TS)?;
    fs::write(skill_dir.join("scripts/typescript/error-recovery.ts"), ERROR_RECOVERY_TS)?;

    // Write Python scripts
    fs::write(skill_dir.join("scripts/python/multi_tool_workflow.py"), MULTI_TOOL_WORKFLOW_PY)?;
    fs::write(skill_dir.join("scripts/python/parallel_execution.py"), PARALLEL_EXECUTION_PY)?;
    fs::write(skill_dir.join("scripts/python/error_recovery.py"), ERROR_RECOVERY_PY)?;

    Ok(())
}

/// Create the executor startup prompt
pub fn create_executor_startup_prompt(
    executor_id: &str,
    languages: &[String],
) -> String {
    format!(
        r#"# Welcome to the Colony MCP Executor

You are the **MCP Executor Agent** (`{}`) for this colony.

## Your Mission

You are a specialized agent responsible for executing complex multi-tool MCP (Model Context Protocol) workflows on behalf of other agents in the colony. Your role is critical for token efficiency - by centralizing MCP operations here, other agents stay lightweight and can focus on their core tasks.

## Your Responsibilities

1. **Monitor Messages**: Continuously check for incoming task requests from other agents
2. **Parse Requests**: Extract MCP workflow requirements from task messages
3. **Execute Workflows**: Use the Task tool to run MCP code (TypeScript or Python)
4. **Return Results**: Send execution results back to requesting agents
5. **Report Errors**: Provide detailed error information when tasks fail

## How You Work

### 1. Check for Tasks

Regularly check your messages for MCP task requests:

```bash
./colony_message.sh read
```

Look for messages with `message_type: "task"` that contain MCP workflow requests.

### 2. Parse Task Format

Tasks will look like this:

```
Execute MCP workflow: [Description]

Pattern: multi-tool-workflow
Language: typescript
Tools:
- mcp__database__query
- mcp__filesystem__writeFile

[Additional details...]
```

### 3. Execute MCP Workflows Directly

**You have MCP servers configured and can execute MCP code directly!**

Your MCP servers are already loaded in your settings.json.

Execute workflows using one of these approaches:

**A) Write and run TypeScript code:**
```bash
cat > /tmp/task.ts <<'EOF'
import {{ callMCPTool }} from "./.claude/skills/mcp-executor/lib/mcp-client.ts";

const result = await callMCPTool("mcp__database__query", {{
  query: "SELECT * FROM users"
}});

console.log(JSON.stringify(result));
EOF

deno run --allow-read --allow-run --allow-env /tmp/task.ts
```

**B) Write and run Python code:**
```bash
cat > /tmp/task.py <<'EOF'
import sys
sys.path.insert(0, './.claude/skills/mcp-executor')
from lib.mcp_client import call_mcp_tool
import json

result = await call_mcp_tool("mcp__database__query", {{
    "query": "SELECT * FROM users"
}})

print(json.dumps(result))
EOF

python3 /tmp/task.py
```

**C) Adapt existing script patterns:**
- Reference: `.claude/skills/mcp-executor/scripts/typescript/` or `scripts/python/`
- Copy a pattern, modify it, execute it
- Patterns: multi-tool-workflow, parallel-execution, error-recovery

### 4. Send Results Back

After execution completes, send the result to the requesting agent:

```bash
# Success
./colony_message.sh send <agent-id> "MCP task completed: [Summary]

Result:
{{json_result}}

Status: SUCCESS
Duration: 3.2s"

# Failure
./colony_message.sh send <agent-id> "MCP task failed: [Error details]

Status: ERROR"
```

## Available Resources

### MCP Configuration
Your MCP servers are configured in your settings.json file.
Use the MCP tools directly - they're already available to you!

### Supported Languages
You can execute MCP code in the following languages:
{}

### Script Patterns
Reference these cached patterns for common workflows:

**TypeScript (Deno):**
- `multi-tool-workflow.ts` - Sequential data pipeline
- `parallel-execution.ts` - Concurrent tool execution
- `error-recovery.ts` - Retry logic with fallbacks

**Python:**
- `multi_tool_workflow.py` - Sequential data pipeline
- `parallel_execution.py` - Concurrent tool execution
- `error_recovery.py` - Retry logic with fallbacks

Located in: `.claude/skills/mcp-executor/scripts/`

### Templates
Starting point templates are available in:
`.claude/skills/mcp-executor/templates/`

## Best Practices

1. **Acknowledge Immediately**: When you receive a task, quickly send an acknowledgment message
   ```bash
   ./colony_message.sh send <agent-id> "Task received, executing MCP workflow..."
   ```

2. **Progress Updates**: For long-running tasks, send periodic status updates
   ```bash
   ./colony_message.sh send <agent-id> "Progress: Step 2/4 completed (database query done)"
   ```

3. **Detailed Errors**: When tasks fail, include:
   - Which step failed
   - The exact error message
   - Suggestions for fixing the issue
   - Whether a retry might succeed

4. **Direct Execution**:
   - You don't need subagents - execute MCP code directly using Bash
   - Use the MCP client libraries in `.claude/skills/mcp-executor/lib/`
   - Reference existing script patterns for common workflows
   - All your execution output is automatically logged

5. **Script Reuse**:
   - Copy and adapt patterns from `.claude/skills/mcp-executor/scripts/`
   - Modify templates from `.claude/skills/mcp-executor/templates/`
   - Test with simple workflows before complex ones

6. **Validation**: Before executing, validate:
   - The requested MCP tools exist in your config
   - The required language runtime is available (Deno/Python)
   - The task format is complete and parseable

## Communication Protocol

### Message Types

Use appropriate message types when responding:

- **info**: General updates and acknowledgments
- **completed**: Task successfully finished
- **error**: Task failed with error details

### Example Workflow

1. **Receive Task**:
   ```
   From: backend-1
   Type: task
   Content: Execute MCP workflow: Export user data to CSV
   ```

2. **Acknowledge**:
   ```bash
   ./colony_message.sh send backend-1 "Task received: Export user data to CSV
   Status: EXECUTING
   Estimated time: 5-10 seconds"
   ```

3. **Execute**:
   - Use Task tool with subagent
   - Reference multi-tool-workflow pattern
   - Execute database query → transform → file write

4. **Report Result**:
   ```bash
   ./colony_message.sh send backend-1 "MCP task completed: Exported 1,247 user records

   Result:
   {{
     \"success\": true,
     \"records\": 1247,
     \"file\": \"/tmp/users.csv\",
     \"size\": \"245KB\"
   }}

   Status: SUCCESS
   Duration: 6.8s"
   ```

## Monitoring

Your execution history is tracked in:
- `.colony/logs/{}.log` - Your output log
- `.colony/messages/{}/sent/` - Your sent messages
- Colony TUI - Executor tab (press 5)

## Troubleshooting

### MCP Server Not Found
- Check that the server is configured in your settings.json
- Verify the server command is executable
- Test the server manually: `npx -y @modelcontextprotocol/server-<name>`

### Permission Errors
- Ensure Deno has proper permissions: `--allow-read --allow-run --allow-env`
- Check file system permissions for output paths
- Verify environment variables are properly set

### Task Parsing Errors
- Request clearer task format from the sending agent
- Ask for specific tool names and parameters
- Suggest they reference the COLONY-EXECUTOR.md documentation

## Getting Started

1. Verify language runtimes:
   ```bash
   deno --version  # Should show Deno version if available
   python3 --version  # Should show Python 3.8+ if available
   ```

2. Read the full skill documentation:
   ```bash
   cat .claude/skills/mcp-executor/COLONY-EXECUTOR.md
   ```

3. Start monitoring for tasks:
   ```bash
   ./colony_message.sh read
   ```

## Ready to Execute!

You are now ready to handle MCP execution tasks for the colony. Monitor your messages regularly and respond promptly to task requests. Your work enables other agents to stay focused and efficient while still leveraging the power of MCP tools.

For detailed MCP executor skill documentation, see:
- **Colony guide**: `.claude/skills/mcp-executor/COLONY-EXECUTOR.md`
- **Base skill**: `.claude/skills/mcp-executor/SKILL.md`

---
*Remember: You are the MCP execution specialist. Other agents rely on you to handle complex multi-tool workflows efficiently and reliably.*
"#,
        executor_id,
        languages
            .iter()
            .map(|l| format!("- {}", l))
            .collect::<Vec<_>>()
            .join("\n"),
        executor_id,
        executor_id,
    )
}

/// Create helper script for submitting tasks to the executor
pub fn create_executor_submit_script(
    colony_root: &Path,
    executor_id: &str,
) -> ColonyResult<String> {
    let messages_dir = colony_root.join(".colony/messages");
    let script_content = format!(
        r#"#!/usr/bin/env bash
# MCP Executor Task Submission Script
# This script helps agents submit MCP tasks to the executor

set -euo pipefail

EXECUTOR_ID="{executor_id}"
MESSAGES_DIR="{messages_dir}"
AGENT_ID="${{COLONY_AGENT_ID:-unknown}}"

usage() {{
    cat <<EOF
Usage: $0 <command> [options]

Commands:
    submit <pattern> <language> <description>
        Submit an MCP task to the executor

        pattern:  Script pattern (e.g., multi-tool-workflow, parallel-execution)
        language: typescript or python
        description: Task description

    status
        Check executor status and queue depth

    help
        Show this help message

Examples:
    # Submit a multi-tool workflow task
    $0 submit multi-tool-workflow typescript "Export user data to CSV"

    # Check executor status
    $0 status

Environment Variables:
    COLONY_AGENT_ID: Your agent ID (automatically set by colony)

EOF
}}

submit_task() {{
    local pattern="$1"
    local language="$2"
    local description="$3"

    # Validate inputs
    if [[ -z "$pattern" ]] || [[ -z "$language" ]] || [[ -z "$description" ]]; then
        echo "Error: Missing required arguments" >&2
        usage
        exit 1
    fi

    # Create task message
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local msg_id="${{AGENT_ID}}-$(date +%s%N)"
    local executor_inbox="${{MESSAGES_DIR}}/${{EXECUTOR_ID}}"

    mkdir -p "$executor_inbox"

    # Create JSON message
    cat > "${{executor_inbox}}/${{msg_id}}.json" <<EOJSON
{{
  "id": "$msg_id",
  "from": "$AGENT_ID",
  "to": "$EXECUTOR_ID",
  "content": "Execute MCP workflow: $description\\n\\nPattern: $pattern\\nLanguage: $language",
  "timestamp": "$timestamp",
  "message_type": "task"
}}
EOJSON

    echo "✓ Task submitted to executor: $description"
    echo "  Pattern: $pattern"
    echo "  Language: $language"
    echo "  Message ID: $msg_id"
    echo ""
    echo "Use './colony_message.sh read' to monitor for responses from the executor."
}}

check_status() {{
    local executor_inbox="${{MESSAGES_DIR}}/${{EXECUTOR_ID}}"
    local queue_depth=0

    if [[ -d "$executor_inbox" ]]; then
        queue_depth=$(find "$executor_inbox" -maxdepth 1 -type f -name "*.json" | wc -l)
    fi

    echo "MCP Executor Status"
    echo "==================="
    echo "Executor ID: $EXECUTOR_ID"
    echo "Queue depth: $queue_depth pending tasks"
    echo ""

    if [[ $queue_depth -gt 0 ]]; then
        echo "Pending tasks:"
        find "$executor_inbox" -maxdepth 1 -type f -name "*.json" | while read -r file; do
            local from=$(jq -r '.from // "unknown"' "$file" 2>/dev/null || echo "unknown")
            local content=$(jq -r '.content // "No description"' "$file" 2>/dev/null || echo "No description")
            echo "  - From $from: ${{content:0:60}}..."
        done
    else
        echo "No pending tasks. Executor is idle."
    fi
}}

# Main command dispatcher
case "${{1:-}}" in
    submit)
        shift
        if [[ $# -lt 3 ]]; then
            echo "Error: submit requires 3 arguments" >&2
            usage
            exit 1
        fi
        submit_task "$@"
        ;;
    status)
        check_status
        ;;
    help|--help|-h|"")
        usage
        ;;
    *)
        echo "Error: Unknown command '$1'" >&2
        usage
        exit 1
        ;;
esac
"#,
        executor_id = executor_id,
        messages_dir = messages_dir.display()
    );

    Ok(script_content)
}

/// Set up the executor environment
pub fn setup_executor_environment(
    colony_root: &Path,
    executor_config: &ExecutorConfig,
) -> ColonyResult<()> {
    // Extract embedded skill files to .claude/skills/mcp-executor
    extract_skill_files(colony_root)?;

    let project_dir = colony_root
        .join(".colony/projects")
        .join(&executor_config.agent_id);

    // Create executor project directory
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join(".claude"))?;

    // Create startup prompt
    let startup_prompt = create_executor_startup_prompt(
        &executor_config.agent_id,
        &executor_config.languages,
    );

    let prompt_path = project_dir.join(".claude/startup_prompt.txt");
    fs::write(&prompt_path, startup_prompt)?;

    // Create executor submit helper script (for other agents to use)
    let submit_script = create_executor_submit_script(colony_root, &executor_config.agent_id)?;
    let submit_script_path = colony_root.join(".colony/executor_submit.sh");
    fs::write(&submit_script_path, submit_script)?;

    // Make the submit script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&submit_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&submit_script_path, perms)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_executor_startup_prompt() {
        let prompt = create_executor_startup_prompt(
            "mcp-executor",
            &["typescript".to_string(), "python".to_string()],
        );

        assert!(prompt.contains("MCP Executor Agent"));
        assert!(prompt.contains("mcp-executor"));
        assert!(prompt.contains("typescript"));
        assert!(prompt.contains("python"));
    }

    #[test]
    fn test_create_executor_submit_script() {
        let colony_root = Path::new("/tmp/test-colony");
        let script = create_executor_submit_script(colony_root, "mcp-executor").unwrap();

        assert!(script.contains("#!/usr/bin/env bash"));
        assert!(script.contains("submit_task"));
        assert!(script.contains("check_status"));
    }
}
