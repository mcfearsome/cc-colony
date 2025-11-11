#!/bin/bash
# Colony Live Dashboard
# Displays real-time colony status, task progress, and activity

COLONY_ROOT="${1:-.colony}"
REFRESH_INTERVAL="${2:-3}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

# Function to count files in directory
count_files() {
    local dir="$1"
    if [ -d "$dir" ]; then
        find "$dir" -maxdepth 1 -type f -name "*.json" 2>/dev/null | wc -l | tr -d ' '
    else
        echo "0"
    fi
}

# Function to get recent messages
get_recent_messages() {
    local messages_dir="$COLONY_ROOT/messages"
    if [ -d "$messages_dir" ]; then
        find "$messages_dir" -name "*.json" -type f -mmin -5 2>/dev/null | \
            xargs -I {} sh -c 'echo "$(stat -f "%Sm" -t "%H:%M:%S" "{}" 2>/dev/null || stat -c "%y" "{}" | cut -d" " -f2 | cut -d"." -f1) $(basename "{}")"' 2>/dev/null | \
            sort -r | head -5
    fi
}

# Function to display dashboard
show_dashboard() {
    clear

    # Header
    echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${CYAN}║${NC}  ${BOLD}COLONY DASHBOARD${NC}                                      ${BOLD}${CYAN}║${NC}"
    echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo

    # Timestamp
    echo -e "${DIM}Last updated: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo

    # Task Statistics
    if [ -d "$COLONY_ROOT/tasks" ]; then
        echo -e "${BOLD}${YELLOW}📋 TASK STATUS${NC}"
        echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        local pending=$(count_files "$COLONY_ROOT/tasks/pending")
        local claimed=$(count_files "$COLONY_ROOT/tasks/claimed")
        local in_progress=$(count_files "$COLONY_ROOT/tasks/in_progress")
        local blocked=$(count_files "$COLONY_ROOT/tasks/blocked")
        local completed=$(count_files "$COLONY_ROOT/tasks/completed")
        local total=$((pending + claimed + in_progress + blocked + completed))

        echo -e "  ${YELLOW}⏳${NC} Pending:     ${pending}"
        echo -e "  ${BLUE}👤${NC} Claimed:     ${claimed}"
        echo -e "  ${CYAN}🔄${NC} In Progress: ${in_progress}"
        echo -e "  ${RED}🚫${NC} Blocked:     ${blocked}"
        echo -e "  ${GREEN}✅${NC} Completed:   ${completed}"
        echo

        if [ $total -gt 0 ]; then
            local completion_pct=$((completed * 100 / total))
            local bar_width=40
            local filled=$((completion_pct * bar_width / 100))
            local empty=$((bar_width - filled))

            echo -n "  Progress: ["
            printf "%${filled}s" | tr ' ' '█'
            printf "%${empty}s" | tr ' ' '░'
            echo -e "] ${completion_pct}%"
        fi
        echo
    fi

    # Agent Status
    if [ -d "$COLONY_ROOT/worktrees" ]; then
        echo -e "${BOLD}${GREEN}👥 ACTIVE AGENTS${NC}"
        echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        # Pre-build agent->task mapping (O(M) instead of O(N*M))
        declare -A agent_tasks
        if [ -d "$COLONY_ROOT/tasks/in_progress" ]; then
            for task_file in "$COLONY_ROOT/tasks/in_progress"/*.json; do
                [ -e "$task_file" ] || continue
                local claimed_by=$(grep -o '"claimed_by": *"[^"]*"' "$task_file" 2>/dev/null | sed 's/"claimed_by": *"\([^"]*\)"/\1/' | head -1)
                if [ -n "$claimed_by" ]; then
                    agent_tasks["$claimed_by"]=$(basename "$task_file" .json)
                fi
            done
        fi

        # Display agents with O(1) task lookup
        for agent_dir in "$COLONY_ROOT/worktrees"/*; do
            if [ -d "$agent_dir" ]; then
                local agent_id=$(basename "$agent_dir")
                local agent_task="${agent_tasks[$agent_id]:-}"

                if [ -n "$agent_task" ]; then
                    echo -e "  ${GREEN}●${NC} ${BOLD}${agent_id}${NC} ${DIM}→ working on ${agent_task}${NC}"
                else
                    echo -e "  ${YELLOW}○${NC} ${agent_id} ${DIM}(idle)${NC}"
                fi
            fi
        done
        echo
    fi

    # Recent Activity (Messages)
    if [ -d "$COLONY_ROOT/messages" ]; then
        echo -e "${BOLD}${CYAN}📬 RECENT ACTIVITY${NC}"
        echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        local msg_count=$(find "$COLONY_ROOT/messages" -name "*.json" -type f -mmin -5 2>/dev/null | wc -l | tr -d ' ')

        if [ "$msg_count" -gt 0 ]; then
            find "$COLONY_ROOT/messages" -name "*.json" -type f -mmin -5 2>/dev/null | \
                sort -t/ -k1 -r | head -5 | while read msg_file; do
                    local from=$(grep -o '"from": *"[^"]*"' "$msg_file" 2>/dev/null | sed 's/"from": *"\([^"]*\)"/\1/' | head -1)
                    local to=$(grep -o '"to": *"[^"]*"' "$msg_file" 2>/dev/null | sed 's/"to": *"\([^"]*\)"/\1/' | head -1)
                    local msg_time=$(stat -f "%Sm" -t "%H:%M" "$msg_file" 2>/dev/null || stat -c "%y" "$msg_file" | cut -d" " -f2 | cut -d":" -f1,2)

                    if [ "$to" = "all" ]; then
                        echo -e "  ${DIM}${msg_time}${NC} ${YELLOW}📢${NC} ${from} ${DIM}→ broadcast${NC}"
                    else
                        echo -e "  ${DIM}${msg_time}${NC} ${BLUE}💬${NC} ${from} ${DIM}→ ${to}${NC}"
                    fi
                done
        else
            echo -e "  ${DIM}No recent activity (last 5 minutes)${NC}"
        fi
        echo
    fi

    # System Info
    echo -e "${BOLD}${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${DIM}Refresh: ${REFRESH_INTERVAL}s | Press Ctrl+C to exit${NC}"
}

# Main loop
echo "Starting Colony Dashboard..."
echo "Monitoring: $COLONY_ROOT"
echo "Refresh interval: ${REFRESH_INTERVAL}s"
echo
sleep 2

while true; do
    show_dashboard
    sleep "$REFRESH_INTERVAL"
done
