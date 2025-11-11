use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ForgeResult;

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
        }
    }

    /// Save message to the message queue
    pub fn save(&self, swarm_root: &Path) -> ForgeResult<()> {
        let messages_dir = swarm_root.join("messages");
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
pub fn load_messages_for_agent(swarm_root: &Path, agent_id: &str) -> ForgeResult<Vec<Message>> {
    let inbox_dir = swarm_root.join("messages").join(agent_id);

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
    let broadcast_dir = swarm_root.join("messages").join("broadcast");
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
pub fn load_all_messages(swarm_root: &Path) -> ForgeResult<Vec<Message>> {
    let messages_dir = swarm_root.join("messages");

    if !messages_dir.exists() {
        return Ok(Vec::new());
    }

    let mut messages = Vec::new();

    // Recursively walk all message directories
    fn walk_messages(dir: &Path, messages: &mut Vec<Message>) -> ForgeResult<()> {
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
pub fn create_message_helper_script(swarm_root: &Path, agent_id: &str) -> ForgeResult<PathBuf> {
    let script_path = swarm_root
        .join("projects")
        .join(agent_id)
        .join("swarm_message.sh");

    // Shell-escape values to prevent injection
    let escaped_root = shell_escape_for_script(&swarm_root.display().to_string());
    let escaped_agent = shell_escape_for_script(agent_id);

    let script_content = format!(
        r#"#!/bin/bash
# Swarm Messaging Helper Script for '{}'
# This script helps agents communicate with each other

SWARM_ROOT='{}'
AGENT_ID='{}'

case "$1" in
    send)
        # Usage: ./swarm_message.sh send <recipient> <message>
        RECIPIENT="$2"
        MESSAGE="$3"
        if [ -z "$RECIPIENT" ] || [ -z "$MESSAGE" ]; then
            echo "Usage: ./swarm_message.sh send <recipient> <message>"
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

        mkdir -p "$SWARM_ROOT/messages/$RECIPIENT"
        mkdir -p "$SWARM_ROOT/messages/$AGENT_ID/sent"

        # Use jq to safely construct JSON (prevents injection)
        if command -v jq >/dev/null 2>&1; then
            jq -n \
                --arg id "$MSG_ID" \
                --arg from "$AGENT_ID" \
                --arg to "$RECIPIENT" \
                --arg content "$MESSAGE" \
                --arg timestamp "$TIMESTAMP" \
                '{{
                    id: $id,
                    from: $from,
                    to: $to,
                    content: $content,
                    timestamp: $timestamp,
                    message_type: "info"
                }}' > "$SWARM_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json"
        else
            # Fallback: use Python for proper JSON escaping if available
            if command -v python3 >/dev/null 2>&1; then
                python3 -c "import json; print(json.dumps({{
                    'id': '''$MSG_ID''',
                    'from': '''$AGENT_ID''',
                    'to': '''$RECIPIENT''',
                    'content': '''$MESSAGE''',
                    'timestamp': '''$TIMESTAMP''',
                    'message_type': 'info'
                }}, indent=2))" > "$SWARM_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json"
            else
                # Last resort: manual JSON escaping with improved handling
                # Escape backslashes first (must be first), then quotes, then newlines
                ESCAPED_MSG=$(printf '%s' "$MESSAGE" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')
                # Use printf to avoid issues with special characters in echo
                printf '{{
  "id": "%s",
  "from": "%s",
  "to": "%s",
  "content": "%s",
  "timestamp": "%s",
  "message_type": "info"
}}\n' "$MSG_ID" "$AGENT_ID" "$RECIPIENT" "$ESCAPED_MSG" "$TIMESTAMP" > "$SWARM_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json"
            fi
        fi

        cp "$SWARM_ROOT/messages/$RECIPIENT/${{MSG_ID}}.json" "$SWARM_ROOT/messages/$AGENT_ID/sent/"
        echo "Message sent to $RECIPIENT"
        ;;

    read)
        # Usage: ./swarm_message.sh read
        echo "=== Messages for {} ==="
        if [ -d "$SWARM_ROOT/messages/$AGENT_ID" ]; then
            for msg in "$SWARM_ROOT/messages/$AGENT_ID"/*.json; do
                [ -e "$msg" ] || continue
                echo "---"
                if command -v jq >/dev/null 2>&1; then
                    cat "$msg" | jq -r '"From: \(.from) | \(.content)"'
                else
                    # Fallback: simple grep-based parsing
                    from=$(grep -o '"from": *"[^"]*"' "$msg" | sed 's/"from": *"\([^"]*\)"/\1/')
                    content=$(grep -o '"content": *"[^"]*"' "$msg" | sed 's/"content": *"\([^"]*\)"/\1/')
                    echo "From: $from | $content"
                fi
            done
        fi

        # Also check broadcast messages
        if [ -d "$SWARM_ROOT/messages/broadcast" ]; then
            for msg in "$SWARM_ROOT/messages/broadcast"/*.json; do
                [ -e "$msg" ] || continue
                echo "---"
                if command -v jq >/dev/null 2>&1; then
                    cat "$msg" | jq -r '"[BROADCAST] From: \(.from) | \(.content)"'
                else
                    # Fallback: simple grep-based parsing
                    from=$(grep -o '"from": *"[^"]*"' "$msg" | sed 's/"from": *"\([^"]*\)"/\1/')
                    content=$(grep -o '"content": *"[^"]*"' "$msg" | sed 's/"content": *"\([^"]*\)"/\1/')
                    echo "[BROADCAST] From: $from | $content"
                fi
            done
        fi
        ;;

    list-agents)
        # Usage: ./swarm_message.sh list-agents
        echo "Active agents in swarm:"
        ls -1 "$SWARM_ROOT/worktrees" 2>/dev/null || echo "No agents found"
        ;;

    *)
        echo "Swarm Messaging Helper"
        echo "Usage:"
        echo "  ./swarm_message.sh send <recipient> <message>  - Send a message"
        echo "  ./swarm_message.sh read                        - Read your messages"
        echo "  ./swarm_message.sh list-agents                 - List all agents"
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

    Ok(script_path)
}

/// Create a README for agents explaining the messaging system
pub fn create_messaging_readme(swarm_root: &Path) -> ForgeResult<()> {
    let readme_path = swarm_root.join("SWARM_COMMUNICATION.md");

    let content = r#"# Forge Swarm Communication Guide

This swarm uses a message queue system for inter-agent communication.

## Messaging Structure

Messages are stored in `.forge-swarm/messages/` with the following structure:

```
.forge-swarm/messages/
├── agent-1/              # Inbox for agent-1
│   └── message-id.json
├── agent-2/              # Inbox for agent-2
│   └── message-id.json
└── broadcast/            # Broadcast messages (visible to all)
    └── message-id.json
```

## How to Communicate

### Using the Helper Script

Each agent has a `swarm_message.sh` script in their project directory:

```bash
# Send a message to another agent
./swarm_message.sh send backend-1 "API endpoints are ready for integration"

# Read your messages
./swarm_message.sh read

# List all agents in the swarm
./swarm_message.sh list-agents
```

### Using Direct File Operations

You can also read/write messages directly. Replace `backend-1` with your actual agent ID:

```bash
# Check your inbox (replace 'backend-1' with your agent ID)
ls .forge-swarm/messages/backend-1/

# Read a specific message
cat .forge-swarm/messages/backend-1/message-id.json

# Check broadcast messages (visible to all agents)
ls .forge-swarm/messages/broadcast/
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

1. **Check messages regularly**: Run `./swarm_message.sh read` periodically
2. **Be specific**: Include context in your messages
3. **Use task type for coordination**: Use message_type "task" for work assignments
4. **Broadcast important updates**: Send to "all" for swarm-wide announcements
5. **Clean communication**: Keep messages concise and actionable

## Coordination Patterns

### Task Claiming
Post a message when you start working on something to avoid duplicate work:

```bash
./swarm_message.sh send all "Starting work on user authentication module"
```

### Asking for Help
```bash
./swarm_message.sh send backend-1 "Need API endpoint spec for user profile"
```

### Sharing Findings
```bash
./swarm_message.sh send all "Found bug in payment processing - investigating"
```

### Completion Notification
```bash
./swarm_message.sh send frontend-1 "API endpoints deployed and tested - ready for integration"
```
"#;

    fs::write(readme_path, content)?;

    Ok(())
}
