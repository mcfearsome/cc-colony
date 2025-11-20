---
name: bash-scripting
description: Bash scripting best practices for Colony agents
---

# Bash Scripting for Colony Agents

## Purpose

Write reliable, maintainable bash scripts for automation, file processing, and tool integration within Colony.

## Script Template

```bash
#!/bin/bash
set -euo pipefail  # Exit on error, undefined vars, pipe failures
IFS=$'\n\t'        # Safer word splitting

# Script purpose and usage
readonly SCRIPT_NAME=$(basename "$0")

usage() {
    cat <<EOF
Usage: $SCRIPT_NAME [OPTIONS] <args>

Description: Brief description here

Options:
    -h, --help     Show this help message
    -v, --verbose  Enable verbose output

Examples:
    $SCRIPT_NAME file.txt
EOF
}

# Main script logic here
main() {
    # Your code
    echo "Script executed successfully"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            set -x
            shift
            ;;
        *)
            break
            ;;
    esac
done

main "$@"
```

## Safety Practices

### Always Use Set Flags

```bash
#!/bin/bash
set -euo pipefail

# -e: Exit immediately if command fails
# -u: Exit if undefined variable is used
# -o pipefail: Pipeline fails if any command fails
```

### Quote Variables

```bash
# ❌ Bad - word splitting issues
cd $DIR
rm $FILE

# ✅ Good - properly quoted
cd "$DIR"
rm "$FILE"

# ✅ Good - array for multiple items
files=("file1" "file 2" "file3")
for file in "${files[@]}"; do
    process "$file"
done
```

### Check Command Existence

```bash
if ! command -v jq &> /dev/null; then
    echo "Error: jq not found. Install with: brew install jq"
    exit 1
fi
```

## Common Patterns

### File Processing

```bash
# Process each line
while IFS= read -r line; do
    echo "Processing: $line"
done < input.txt

# Find and process files
find . -name "*.ts" -type f -print0 | while IFS= read -r -d '' file; do
    echo "Found: $file"
done
```

### Error Handling

```bash
# Function with error handling
process_file() {
    local file=$1

    if [[ ! -f "$file" ]]; then
        echo "Error: File not found: $file" >&2
        return 1
    fi

    if ! grep -q "pattern" "$file"; then
        echo "Warning: Pattern not found in $file" >&2
        return 0
    fi

    # Process file
    return 0
}

# Use in script
if process_file "myfile.txt"; then
    echo "Success"
else
    echo "Failed with exit code: $?"
fi
```

### Logging

```bash
# Log levels
log_info() {
    echo "[INFO] $*"
}

log_error() {
    echo "[ERROR] $*" >&2
}

log_debug() {
    [[ "${DEBUG:-0}" == "1" ]] && echo "[DEBUG] $*"
}

# Usage
log_info "Starting process..."
log_error "Failed to connect"
DEBUG=1 log_debug "Variable value: $VAR"
```

### Temporary Files

```bash
# Create temp file safely
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT  # Cleanup on exit

echo "data" > "$TMPFILE"
process "$TMPFILE"
# File automatically deleted on exit
```

### JSON Processing

```bash
# With jq
data=$(curl -s api.example.com/data)
name=$(echo "$data" | jq -r '.name')
items=$(echo "$data" | jq -r '.items[]')

# Without jq (grep/sed fallback)
name=$(echo "$data" | grep -o '"name": *"[^"]*"' | sed 's/"name": *"\([^"]*\)"/\1/')
```

## Integration with Colony

### Script that Sends Messages

```bash
#!/bin/bash
set -euo pipefail

# Process files and report results
process_directory() {
    local dir=$1
    local count=0

    for file in "$dir"/*.ts; do
        [[ -e "$file" ]] || continue
        # Process file
        ((count++))
    done

    # Report via colony messaging
    ./colony_message.sh send orchestrator "Processed $count TypeScript files in $dir"
}

process_directory "src/components"
```

### Script that Reads Messages

```bash
#!/bin/bash
# Check for task assignments in messages

MESSAGES=$(./colony_message.sh read | grep -i "task:")

if [[ -n "$MESSAGES" ]]; then
    echo "Found task assignments:"
    echo "$MESSAGES"

    # Parse and execute
    # (Add your task parsing logic)
fi
```

### Pane Management Script

```bash
#!/bin/bash
# Helper to manage tool panes

SESSION=$(tmux display-message -p '#{session_name}')
MY_PANE=$(tmux display-message -p '#{pane_index}')

create_tool_pane() {
    local tool=$1
    tmux split-window -v -t "$SESSION:0.$MY_PANE" "$tool"
    tmux display-message -p '#{pane_index}' -t "$SESSION:0.{last}"
}

# Usage
NVIM_PANE=$(create_tool_pane "nvim")
echo "Created nvim in pane $NVIM_PANE"
```

## Testing Scripts

### Dry Run Mode

```bash
DRY_RUN=${DRY_RUN:-0}

run_command() {
    if [[ "$DRY_RUN" == "1" ]]; then
        echo "[DRY RUN] Would execute: $*"
    else
        "$@"
    fi
}

# Usage
run_command rm -rf dangerous/path
# With: DRY_RUN=1 ./script.sh  (safe testing)
```

### Debug Mode

```bash
# Enable with DEBUG=1
debug() {
    [[ "${DEBUG:-0}" == "1" ]] && echo "[DEBUG] $*" >&2
}

debug "Variable state: VAR=$VAR"
debug "About to process file: $file"
```

## Performance

### Parallel Processing

```bash
# Process files in parallel
find . -name "*.ts" -print0 | xargs -0 -P 4 -I {} bash -c 'process_file "{}"'

# With function
export -f process_file
find . -name "*.ts" -print0 | xargs -0 -P 4 -n 1 bash -c 'process_file "$@"' _
```

### Efficient File Reading

```bash
# ❌ Slow - creates subshell per line
cat file.txt | while read line; do
    process "$line"
done

# ✅ Fast - no subshell
while IFS= read -r line; do
    process "$line"
done < file.txt
```

## Common Pitfalls

### Word Splitting

```bash
# ❌ Fails with spaces in filename
for file in $(find . -name "*.ts"); do
    echo "$file"
done

# ✅ Correct - handles spaces
find . -name "*.ts" -print0 | while IFS= read -r -d '' file; do
    echo "$file"
done
```

### Variable Expansion

```bash
# ❌ Wrong - literal text
greeting='Hello $name'

# ✅ Correct - use double quotes for expansion
greeting="Hello $name"

# ✅ Correct - prevent expansion with single quotes
literal='$VAR'  # Stays as $VAR
```

### Exit Codes

```bash
# Check exit code properly
if command arg; then
    echo "Success"
else
    echo "Failed with code: $?"
fi

# Or capture but still check
output=$(command arg) || {
    echo "Command failed"
    exit 1
}
```

## Shell Best Practices

**1. Use `[[` over `[`**
```bash
# ✅ Better - handles empty vars, supports &&, ||
if [[ -f "$file" && -r "$file" ]]; then
    echo "File exists and is readable"
fi
```

**2. Use `$()` over backticks**
```bash
# ✅ Better - nestable, clearer
result=$(command $(inner_command))
```

**3. Prefer functions over inline code**
```bash
# ✅ Reusable and testable
validate_input() {
    local input=$1
    [[ -n "$input" ]] && [[ "$input" =~ ^[a-zA-Z0-9_-]+$ ]]
}

if validate_input "$USER_INPUT"; then
    process "$USER_INPUT"
fi
```

## Useful Utilities

### String Manipulation

```bash
# Lowercase
lowercase="${string,,}"

# Uppercase
uppercase="${string^^}"

# Remove prefix/suffix
filename="${path##*/}"   # basename
extension="${filename##*.}"
name="${filename%.*}"

# Replace
new_string="${string/old/new}"    # First occurrence
new_string="${string//old/new}"   # All occurrences
```

### Arrays

```bash
# Declare array
files=("file1.ts" "file2.ts" "file3.ts")

# Append
files+=("file4.ts")

# Length
echo "Count: ${#files[@]}"

# Iterate
for file in "${files[@]}"; do
    echo "$file"
done

# Slice
subset=("${files[@]:1:2}")  # Elements 1-2
```

### Process Management

```bash
# Run in background
command &
pid=$!

# Wait for completion
wait $pid

# Check if running
if kill -0 $pid 2>/dev/null; then
    echo "Process $pid is running"
fi
```

## Colony-Specific Patterns

### Report Progress

```bash
#!/bin/bash
set -euo pipefail

total_files=$(find src/ -name "*.ts" | wc -l)
processed=0

find src/ -name "*.ts" -type f | while read -r file; do
    # Process file
    lint "$file"

    ((processed++))

    # Report every 10 files
    if (( processed % 10 == 0 )); then
        ./colony_message.sh send orchestrator "Progress: $processed/$total_files files processed"
    fi
done

./colony_message.sh send orchestrator "Completed: Processed $total_files TypeScript files"
```

### Coordinate with Other Agents

```bash
#!/bin/bash
# Wait for another agent to complete their task

check_for_completion() {
    ./colony_message.sh read | grep -q "backend-1.*API ready"
}

echo "Waiting for backend-1 to complete API..."

while ! check_for_completion; do
    sleep 5
done

echo "Backend ready! Starting frontend work..."
./colony_message.sh send backend-1 "Received your API completion. Starting frontend integration."
```

## Debugging

### ShellCheck

```bash
# Check script for common issues
shellcheck myscript.sh

# Ignore specific warnings
# shellcheck disable=SC2086
command $unquoted_var  # Sometimes intentional
```

### Trace Execution

```bash
# Show each command before executing
bash -x script.sh

# Or within script
set -x     # Enable tracing
command
set +x     # Disable tracing
```

### Common Debug Info

```bash
debug_info() {
    echo "=== Debug Info ==="
    echo "PWD: $PWD"
    echo "USER: $USER"
    echo "SHELL: $SHELL"
    echo "PATH: $PATH"
    echo "Args: $*"
    echo "=================="
}
```

## Resources

- Use `man bash` for built-in documentation
- ShellCheck for linting: https://www.shellcheck.net/
- Bash Guide: https://mywiki.wooledge.org/BashGuide
