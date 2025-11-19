#!/bin/bash
# Sync colony data to manifest backend
# This script demonstrates how colony data could be synced to the web UI

set -e

COLONY_DIR="${COLONY_DIR:-.colony}"
MANIFEST_API="${MANIFEST_API:-http://localhost:1111/api}"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🐝 Syncing colony data to manifest backend...${NC}"

# Check if manifest is running
if ! curl -s "$MANIFEST_API/tasks" > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: Manifest backend not running at $MANIFEST_API${NC}"
    echo "Start it with: manifest dev"
    exit 1
fi

# Check if colony directory exists
if [ ! -d "$COLONY_DIR" ]; then
    echo -e "${RED}❌ Error: Colony directory not found: $COLONY_DIR${NC}"
    exit 1
fi

# Sync tasks
if [ -f "$COLONY_DIR/tasks.json" ]; then
    echo -e "${YELLOW}📋 Syncing tasks...${NC}"
    task_count=0

    # Parse tasks.json and post each task
    jq -c '.[]?' "$COLONY_DIR/tasks.json" 2>/dev/null | while read -r task; do
        if [ -n "$task" ]; then
            response=$(curl -s -X POST "$MANIFEST_API/tasks" \
                 -H "Content-Type: application/json" \
                 -d "$task")

            if [ $? -eq 0 ]; then
                task_count=$((task_count + 1))
                task_id=$(echo "$task" | jq -r '.id // "unknown"')
                echo -e "  ✓ Synced task: $task_id"
            else
                echo -e "  ${RED}✗ Failed to sync task${NC}"
            fi
        fi
    done

    echo -e "${GREEN}✓ Synced tasks${NC}"
else
    echo -e "${YELLOW}⚠ No tasks.json found, skipping${NC}"
fi

# Sync agents
if [ -f "$COLONY_DIR/agents.json" ]; then
    echo -e "${YELLOW}🤖 Syncing agents...${NC}"
    agent_count=0

    jq -c '.[]?' "$COLONY_DIR/agents.json" 2>/dev/null | while read -r agent; do
        if [ -n "$agent" ]; then
            response=$(curl -s -X POST "$MANIFEST_API/agents" \
                 -H "Content-Type: application/json" \
                 -d "$agent")

            if [ $? -eq 0 ]; then
                agent_count=$((agent_count + 1))
                agent_id=$(echo "$agent" | jq -r '.id // "unknown"')
                echo -e "  ✓ Synced agent: $agent_id"
            else
                echo -e "  ${RED}✗ Failed to sync agent${NC}"
            fi
        fi
    done

    echo -e "${GREEN}✓ Synced agents${NC}"
else
    echo -e "${YELLOW}⚠ No agents.json found, skipping${NC}"
fi

# Sync messages
if [ -f "$COLONY_DIR/messages.json" ]; then
    echo -e "${YELLOW}💬 Syncing messages...${NC}"
    message_count=0

    jq -c '.[]?' "$COLONY_DIR/messages.json" 2>/dev/null | while read -r message; do
        if [ -n "$message" ]; then
            response=$(curl -s -X POST "$MANIFEST_API/messages" \
                 -H "Content-Type: application/json" \
                 -d "$message")

            if [ $? -eq 0 ]; then
                message_count=$((message_count + 1))
                echo -e "  ✓ Synced message"
            else
                echo -e "  ${RED}✗ Failed to sync message${NC}"
            fi
        fi
    done

    echo -e "${GREEN}✓ Synced messages${NC}"
else
    echo -e "${YELLOW}⚠ No messages.json found, skipping${NC}"
fi

echo -e "${GREEN}✨ Sync complete!${NC}"
echo -e "Open ${YELLOW}http://localhost:1111${NC} to view the UI"
