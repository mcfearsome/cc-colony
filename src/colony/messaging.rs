use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ColonyResult;

/// A message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID (timestamp-based)
    pub id: String,
    /// Sender agent ID
    pub from: String,
    /// Recipient agent ID (or "all" for broadcast)
    pub to: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: String,
    /// Message type
    #[serde(default = "default_message_type")]
    pub message_type: MessageType,
    /// Project directory context (working directory of sender)
    #[serde(default)]
    pub project_dir: Option<String>,
    /// Git branch context
    #[serde(default)]
    pub git_branch: Option<String>,
}

fn default_message_type() -> MessageType {
    MessageType::Info
}

/// Type of message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    /// Informational message
    Info,
    /// Task assignment or request
    Task,
    /// Question to another agent
    Question,
    /// Response to a question
    Answer,
    /// Work completed notification
    Completed,
    /// Error or blocker
    Error,
}

impl Message {
    /// Create a new message
    pub fn new(from: &str, to: &str, content: String, message_type: MessageType) -> Self {
        let now = Utc::now();
        let timestamp = now.to_rfc3339();
        // Include nanoseconds to prevent collisions if multiple messages sent in same second
        let id = format!(
            "{}-{}-{}",
            from,
            now.timestamp(),
            now.timestamp_subsec_nanos()
        );

        Self {
            id,
            from: from.to_string(),
            to: to.to_string(),
            content,
            timestamp,
            message_type,
            project_dir: None,
            git_branch: None,
        }
    }

    /// Save message to the message queue
    pub fn save(&self, colony_root: &Path) -> ColonyResult<()> {
        let messages_dir = colony_root.join("messages");
        fs::create_dir_all(&messages_dir)?;

        // Save to recipient's inbox
        let inbox_dir = if self.to == "all" {
            messages_dir.join("broadcast")
        } else {
            messages_dir.join(&self.to)
        };
        fs::create_dir_all(&inbox_dir)?;

        let message_file = inbox_dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(message_file, json)?;

        // Also save to sender's outbox for record keeping
        let outbox_dir = messages_dir.join(&self.from).join("sent");
        fs::create_dir_all(&outbox_dir)?;

        let outbox_file = outbox_dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(outbox_file, json)?;

        Ok(())
    }
}

/// Load all messages for a specific agent
pub fn load_messages_for_agent(colony_root: &Path, agent_id: &str) -> ColonyResult<Vec<Message>> {
    let inbox_dir = colony_root.join("messages").join(agent_id);

    if !inbox_dir.exists() {
        return Ok(Vec::new());
    }

    let mut messages = Vec::new();

    for entry in fs::read_dir(inbox_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            if let Ok(message) = serde_json::from_str::<Message>(&content) {
                messages.push(message);
            }
        }
    }

    // Also load broadcast messages
    let broadcast_dir = colony_root.join("messages").join("broadcast");
    if broadcast_dir.exists() {
        for entry in fs::read_dir(broadcast_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(message) = serde_json::from_str::<Message>(&content) {
                    messages.push(message);
                }
            }
        }
    }

    // Sort by timestamp
    messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(messages)
}

/// Load all messages in the system
pub fn load_all_messages(colony_root: &Path) -> ColonyResult<Vec<Message>> {
    let messages_dir = colony_root.join("messages");

    if !messages_dir.exists() {
        return Ok(Vec::new());
    }

    let mut messages = Vec::new();

    // Recursively walk all message directories
    fn walk_messages(dir: &Path, messages: &mut Vec<Message>) -> ColonyResult<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                walk_messages(&path, messages)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(message) = serde_json::from_str::<Message>(&content) {
                    messages.push(message);
                }
            }
        }

        Ok(())
    }

    walk_messages(&messages_dir, &mut messages)?;

    // Sort by timestamp and deduplicate
    messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    messages.dedup_by(|a, b| a.id == b.id);

    Ok(messages)
}

/// Shell-escape a string for safe embedding in bash scripts
fn shell_escape_for_script(s: &str) -> String {
    // Escape single quotes by replacing ' with '\''
    s.replace('\'', r"'\''")
}

/// Create message helper script for agents
pub fn create_message_helper_script(
    colony_root: &Path,
    agent_id: &str,
    worktree_path: Option<&Path>,
) -> ColonyResult<PathBuf> {
    let script_path = colony_root
        .join("projects")
        .join(agent_id)
        .join("colony_message.sh");

    // Shell-escape values to prevent injection
    let escaped_root = shell_escape_for_script(&colony_root.display().to_string());
    let escaped_agent = shell_escape_for_script(agent_id);

    let script_content = format!(
        r#"#!/bin/bash
# Colony Messaging Helper Script for '{}'
# This script helps agents communicate with each other

COLONY_ROOT='{}'
AGENT_ID='{}'

case "$1" in
    send)
        # Usage: ./colony_message.sh send <recipient> <message>
        RECIPIENT="$2"
        MESSAGE="$3"
        if [ -z "$RECIPIENT" ] || [ -z "$MESSAGE" ]; then
            echo "Usage: ./colony_message.sh send <recipient> <message>"
            exit 1
        fi

        # Validate recipient ID (only alphanumeric, hyphens, underscores, or "all")
        if ! (echo "$RECIPIENT" | grep -qE '^[a-zA-Z0-9_-]+$' || [ "$RECIPIENT" = "all" ]); then
            echo "Error: Invalid recipient ID '$RECIPIENT'"
            echo "Recipient must contain only alphanumeric characters, hyphens, underscores, or 'all'"
            exit 1
        fi

        TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        # Include nanoseconds to prevent ID collisions
        TIMESTAMP_NS=$(date +%s%N 2>/dev/null || date +%s000000000)
        MSG_ID="${{AGENT_ID}}-${{TIMESTAMP_NS}}"

        # Capture context: current directory and git branch
        PROJECT_DIR="$PWD"
        GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")

        mkdir -p "$COLONY_ROOT/messages/$RECIPIENT"
        mkdir -p "$COLONY_ROOT/messages/$AGENT_ID/sent"

        # Use jq to safely construct JSON (prevents injection)
        if command -v jq >/dev/null 2>&1; then
            jq -n \
                --arg id "$MSG_ID" \
                --arg from "$AGENT_ID" \
                --arg to "$RECIPIENT" \
                --arg content "$MESSAGE" \
                --arg timestamp "$TIMESTAMP" \
                --arg project_dir "$PROJECT_DIR" \
                --arg git_branch "$GIT_BRANCH" \
                '{{
                    id: $id,
                    from: $from,
                    to: $to,
                    content: $content,
                    timestamp: $timestamp,
                    message_type: "info",
                    project_dir: (if $project_dir != "" then $project_dir else null end),
                    git_branch: (if $git_branch != "" then $git_branch else null end)
                }}' > "$COLONY_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json"
        else
            # Fallback: use Python for proper JSON escaping if available
            if command -v python3 >/dev/null 2>&1; then
                python3 -c "import json; print(json.dumps({{
                    'id': '''$MSG_ID''',
                    'from': '''$AGENT_ID''',
                    'to': '''$RECIPIENT''',
                    'content': '''$MESSAGE''',
                    'timestamp': '''$TIMESTAMP''',
                    'message_type': 'info',
                    'project_dir': '''$PROJECT_DIR''' if '''$PROJECT_DIR''' else None,
                    'git_branch': '''$GIT_BRANCH''' if '''$GIT_BRANCH''' else None
                }}, indent=2))" > "$COLONY_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json"
            else
                # Last resort: manual JSON escaping with improved handling
                # Escape backslashes first (must be first), then quotes, then newlines
                ESCAPED_MSG=$(printf '%s' "$MESSAGE" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')
                ESCAPED_DIR=$(printf '%s' "$PROJECT_DIR" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g')
                ESCAPED_BRANCH=$(printf '%s' "$GIT_BRANCH" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g')
                # Use printf to avoid issues with special characters in echo
                printf '{{
  "id": "%s",
  "from": "%s",
  "to": "%s",
  "content": "%s",
  "timestamp": "%s",
  "message_type": "info",
  "project_dir": "%s",
  "git_branch": %s
}}\n' "$MSG_ID" "$AGENT_ID" "$RECIPIENT" "$ESCAPED_MSG" "$TIMESTAMP" "$ESCAPED_DIR" "$(if [ -n "$ESCAPED_BRANCH" ]; then echo "\"$ESCAPED_BRANCH\""; else echo "null"; fi)" > "$COLONY_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json"
            fi
        fi

        cp "$COLONY_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json" "$COLONY_ROOT/messages/$AGENT_ID/sent/"
        echo "Message sent to $RECIPIENT"
        ;;

    read)
        # Usage: ./colony_message.sh read
        echo "=== Messages for {} ==="
        if [ -d "$COLONY_ROOT/messages/$AGENT_ID" ]; then
            for msg in "$COLONY_ROOT/messages/$AGENT_ID"/*.json; do
                [ -e "$msg" ] || continue
                echo "---"
                if command -v jq >/dev/null 2>&1; then
                    cat "$msg" | jq -r '
                        "From: \(.from)" +
                        (if .project_dir then " [\(.project_dir)]" else "" end) +
                        (if .git_branch then " (\(.git_branch))" else "" end) +
                        "\n\(.content)"'
                else
                    # Fallback: simple grep-based parsing
                    from=$(grep -o '"from": *"[^"]*"' "$msg" | sed 's/"from": *"\([^"]*\)"/\1/')
                    content=$(grep -o '"content": *"[^"]*"' "$msg" | sed 's/"content": *"\([^"]*\)"/\1/')
                    project_dir=$(grep -o '"project_dir": *"[^"]*"' "$msg" | sed 's/"project_dir": *"\([^"]*\)"/\1/' || echo "")
                    git_branch=$(grep -o '"git_branch": *"[^"]*"' "$msg" | sed 's/"git_branch": *"\([^"]*\)"/\1/' || echo "")
                    echo -n "From: $from"
                    [ -n "$project_dir" ] && echo -n " [$project_dir]"
                    [ -n "$git_branch" ] && echo -n " ($git_branch)"
                    echo ""
                    echo "$content"
                fi
            done
        fi

        # Also check broadcast messages
        if [ -d "$COLONY_ROOT/messages/broadcast" ]; then
            for msg in "$COLONY_ROOT/messages/broadcast"/*.json; do
                [ -e "$msg" ] || continue
                echo "---"
                if command -v jq >/dev/null 2>&1; then
                    cat "$msg" | jq -r '
                        "[BROADCAST] From: \(.from)" +
                        (if .project_dir then " [\(.project_dir)]" else "" end) +
                        (if .git_branch then " (\(.git_branch))" else "" end) +
                        "\n\(.content)"'
                else
                    # Fallback: simple grep-based parsing
                    from=$(grep -o '"from": *"[^"]*"' "$msg" | sed 's/"from": *"\([^"]*\)"/\1/')
                    content=$(grep -o '"content": *"[^"]*"' "$msg" | sed 's/"content": *"\([^"]*\)"/\1/')
                    project_dir=$(grep -o '"project_dir": *"[^"]*"' "$msg" | sed 's/"project_dir": *"\([^"]*\)"/\1/' || echo "")
                    git_branch=$(grep -o '"git_branch": *"[^"]*"' "$msg" | sed 's/"git_branch": *"\([^"]*\)"/\1/' || echo "")
                    echo -n "[BROADCAST] From: $from"
                    [ -n "$project_dir" ] && echo -n " [$project_dir]"
                    [ -n "$git_branch" ] && echo -n " ($git_branch)"
                    echo ""
                    echo "$content"
                fi
            done
        fi
        ;;

    list-agents)
        # Usage: ./colony_message.sh list-agents
        echo "Active agents in colony:"
        ls -1 "$COLONY_ROOT/worktrees" 2>/dev/null || echo "No agents found"
        ;;

    *)
        echo "Colony Messaging Helper"
        echo "Usage:"
        echo "  ./colony_message.sh send <recipient> <message>  - Send a message"
        echo "  ./colony_message.sh read                        - Read your messages"
        echo "  ./colony_message.sh list-agents                 - List all agents"
        exit 1
        ;;
esac
"#,
        agent_id, escaped_root, escaped_agent, agent_id
    );

    fs::write(&script_path, script_content)?;

    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // Create symlink in the worktree if path is provided
    if let Some(worktree_dir) = worktree_path {
        // Create both a generic symlink and an agent-specific one
        // Generic: ./colony_message.sh (may be overwritten in shared worktrees)
        // Specific: ./colony_message_{agent-id}.sh (always unique)

        let generic_symlink = worktree_dir.join("colony_message.sh");
        let specific_symlink = worktree_dir.join(format!("colony_message_{}.sh", agent_id));

        // Create relative path from worktree to project directory
        // From: .colony/worktrees/{name}/colony_message.sh
        // To:   .colony/projects/{agent-id}/colony_message.sh
        // Relative path: ../../projects/{agent-id}/colony_message.sh
        let relative_path = format!("../../projects/{}/colony_message.sh", agent_id);

        #[cfg(unix)]
        {
            // Create agent-specific symlink (always unique)
            if specific_symlink.exists() || specific_symlink.symlink_metadata().is_ok() {
                let _ = std::fs::remove_file(&specific_symlink);
            }
            std::os::unix::fs::symlink(&relative_path, &specific_symlink)
                .map_err(|e| crate::error::ColonyError::Colony(format!(
                    "Failed to create specific symlink for agent '{}' at {}: {}",
                    agent_id,
                    specific_symlink.display(),
                    e
                )))?;

            // Create generic symlink (for convenience when worktree is not shared)
            if generic_symlink.exists() || generic_symlink.symlink_metadata().is_ok() {
                let _ = std::fs::remove_file(&generic_symlink);
            }
            std::os::unix::fs::symlink(&relative_path, &generic_symlink)
                .map_err(|e| crate::error::ColonyError::Colony(format!(
                    "Failed to create generic symlink for agent '{}' at {}: {}",
                    agent_id,
                    generic_symlink.display(),
                    e
                )))?;
        }
    }

    Ok(script_path)
}

/// Create a README for agents explaining the messaging system
pub fn create_messaging_readme(colony_root: &Path) -> ColonyResult<()> {
    let readme_path = colony_root.join("COLONY_COMMUNICATION.md");

    let content = r#"# Colony Communication Guide

This colony uses a message queue system for inter-agent communication.

## Messaging Structure

Messages are stored in `.colony/messages/` with the following structure:

```
.colony/messages/
├── agent-1/              # Inbox for agent-1
│   └── message-id.json
├── agent-2/              # Inbox for agent-2
│   └── message-id.json
└── broadcast/            # Broadcast messages (visible to all)
    └── message-id.json
```

## How to Communicate

### Using the Helper Script

Each agent has a `colony_message.sh` script in their project directory:

```bash
# Send a message to another agent
./colony_message.sh send backend-1 "API endpoints are ready for integration"

# Read your messages
./colony_message.sh read

# List all agents in the colony
./colony_message.sh list-agents
```

### Using Direct File Operations

You can also read/write messages directly. Replace `backend-1` with your actual agent ID:

```bash
# Check your inbox (replace 'backend-1' with your agent ID)
ls .colony/messages/backend-1/

# Read a specific message
cat .colony/messages/backend-1/message-id.json

# Check broadcast messages (visible to all agents)
ls .colony/messages/broadcast/
```

## Message Format

Messages are JSON files with this structure:

```json
{
  "id": "agent-id-timestamp",
  "from": "sender-agent-id",
  "to": "recipient-agent-id",
  "content": "Message content here",
  "timestamp": "2025-01-10T12:00:00Z",
  "message_type": "info"
}
```

## Message Types

- `info`: General information
- `task`: Task assignment or request
- `question`: Question to another agent
- `answer`: Response to a question
- `completed`: Work completion notification
- `error`: Error or blocker notification

## Best Practices

1. **Check messages regularly**: Run `./colony_message.sh read` periodically
2. **Be specific**: Include context in your messages
3. **Use task type for coordination**: Use message_type "task" for work assignments
4. **Broadcast important updates**: Send to "all" for colony-wide announcements
5. **Clean communication**: Keep messages concise and actionable

## Coordination Patterns

### Task Claiming
Post a message when you start working on something to avoid duplicate work:

```bash
./colony_message.sh send all "Starting work on user authentication module"
```

### Asking for Help
```bash
./colony_message.sh send backend-1 "Need API endpoint spec for user profile"
```

### Sharing Findings
```bash
./colony_message.sh send all "Found bug in payment processing - investigating"
```

### Completion Notification
```bash
./colony_message.sh send frontend-1 "API endpoints deployed and tested - ready for integration"
```
"#;

    fs::write(readme_path, content)?;

    Ok(())
}
