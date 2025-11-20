---
name: ollama-local-llm
description: Using Ollama for fast local LLM inference within Colony agents
---

# Ollama Local LLM for Colony Agents

## Purpose

Use Ollama to run local LLMs (like CodeLlama, Llama 3, Mistral) for quick code analysis, suggestions, and testing without API calls or latency.

## When to Use Ollama

**✅ Use Ollama For:**
- Quick code analysis or suggestions
- Bulk operations (analyzing many files)
- Fast iteration on prompts
- Privacy-sensitive code review
- Testing before sending to main LLM
- Simple transformations

**❌ Don't Use Ollama For:**
- Complex reasoning (use your main Claude capabilities)
- Tasks requiring latest knowledge
- Very large context (>32K tokens)
- Critical production code review

## Prerequisites

### Check if Ollama is Installed

```bash
which ollama
# Should return: /usr/local/bin/ollama or similar
```

### Check Available Models

```bash
ollama list
```

Common models:
- `codellama:7b` - Fast code generation
- `llama3:8b` - General purpose, good reasoning
- `mistral:7b` - Fast, efficient
- `qwen2.5-coder:7b` - Excellent for coding

## Basic Usage

### Single Prompt

```bash
# Simple query
ollama run codellama "Explain this function: $(cat file.ts)"

# With specific model
ollama run llama3:8b "Review this code for bugs"
```

### Streaming vs Non-streaming

```bash
# Streaming (default) - see output as it generates
ollama run codellama "prompt here"

# Non-streaming - wait for full response
ollama run --no-stream codellama "prompt here"
```

## Common Patterns

### Code Analysis

```bash
# Analyze a file
ollama run codellama "Analyze this TypeScript code for potential bugs:

$(cat src/component.tsx)

Focus on: type safety, error handling, edge cases"
```

### Quick Suggestions

```bash
# Get function implementation ideas
ollama run codellama "Write a TypeScript function that:
- Takes an array of users
- Filters by active status
- Sorts by name
- Returns formatted list"
```

### Bulk File Analysis

```bash
# Analyze multiple files
for file in src/**/*.ts; do
  echo "=== $file ==="
  ollama run --no-stream codellama "Quick review of $file:
$(cat $file)

List any issues (max 3):"
done
```

### Code Transformation

```bash
# Convert code style
ollama run codellama "Convert this JavaScript to TypeScript:

$(cat legacy.js)

Add proper types."
```

### Test Generation

```bash
# Generate test cases
ollama run codellama "Generate Jest test cases for:

$(cat src/auth.ts)

Cover happy path and error cases."
```

## Using Ollama in a Pane

For longer sessions, run Ollama in a separate pane:

```bash
SESSION=$(tmux display-message -p '#{session_name}')
MY_PANE=$(tmux display-message -p '#{pane_index}')

# Create ollama pane
tmux split-window -v -t $SESSION:0.$MY_PANE
OLLAMA_PANE=$(tmux display-message -p '#{pane_index}' -t $SESSION:0.{last})

# Start interactive ollama session
tmux send-keys -t $SESSION:0.$OLLAMA_PANE "ollama run codellama" C-m
sleep 3

# Send prompt (literal text mode for multi-line)
tmux send-keys -t $SESSION:0.$OLLAMA_PANE -l "Explain this code:
function foo() {
  return bar();
}"
tmux send-keys -t $SESSION:0.$OLLAMA_PANE C-m C-m  # Two enters to submit

# Wait and capture response
sleep 5
tmux capture-pane -t $SESSION:0.$OLLAMA_PANE -p -S -30

# Send another prompt
tmux send-keys -t $SESSION:0.$OLLAMA_PANE -l "Now suggest improvements"
tmux send-keys -t $SESSION:0.$OLLAMA_PANE C-m C-m

# When done
tmux send-keys -t $SESSION:0.$OLLAMA_PANE "/bye" C-m
tmux kill-pane -t $SESSION:0.$OLLAMA_PANE
```

## Model Selection

### Choosing the Right Model

**Speed Priority:**
- `codellama:7b` - Very fast, good for code
- `qwen2.5-coder:7b` - Fast, excellent code understanding
- `mistral:7b` - Fast, general purpose

**Quality Priority:**
- `codellama:13b` - Better reasoning, slower
- `llama3:70b` - Best quality (if you have the RAM/GPU)

**Specialized:**
- `qwen2.5-coder:14b` - Best for code (if resources allow)
- `deepseek-coder:6.7b` - Good code completion

### Pulling New Models

```bash
# Download a model (run once)
ollama pull codellama:7b

# See all available models
ollama list
```

## API Mode (Programmatic Access)

Ollama also provides an API for more control:

```bash
# Using curl for structured responses
curl http://localhost:11434/api/generate -d '{
  "model": "codellama",
  "prompt": "Explain this code",
  "stream": false
}' | jq -r '.response'
```

## Performance Tips

**1. Model Size vs Speed**
- 7B models: Very fast, good enough for most tasks
- 13B models: Slower but better quality
- 70B models: Requires significant resources

**2. Context Length**
- Most models: 4K-8K tokens
- Keep prompts focused
- For large files, extract relevant sections

**3. Caching**
- Ollama caches model in memory after first use
- First run is slow, subsequent runs are fast

**4. GPU Acceleration**
- Ollama automatically uses GPU if available
- Check with: `ollama ps` (shows loaded models)

## Integration with Colony Workflow

### Pattern: Quick Analysis Before Detailed Work

```bash
# 1. Quick check with Ollama
ollama run --no-stream codellama "Is this code safe? $(cat auth.ts)" > /tmp/quick-review.txt

# 2. Read Ollama's assessment
cat /tmp/quick-review.txt

# 3. If issues found, do detailed analysis yourself
# 4. Report findings to other agents via ./colony_message.sh
```

### Pattern: Batch Processing

```bash
# Process multiple files quickly
find src/ -name "*.ts" -type f | while read file; do
  result=$(ollama run --no-stream codellama "Rate code quality 1-5: $(cat $file)")
  echo "$file: $result"
done > code-quality-report.txt

./colony_message.sh send orchestrator "Completed code quality analysis. See code-quality-report.txt"
```

### Pattern: Interactive Pair Programming

```bash
# Keep ollama running in a pane for quick consultations
# You focus on architecture, ollama handles quick syntax questions
tmux split-window -v "ollama run codellama"
OLLAMA_PANE=$(tmux display-message -p '#{pane_index}' -t {last})

# Throughout your work, ask quick questions:
tmux send-keys -t $OLLAMA_PANE -l "TypeScript syntax for async generator?"
tmux send-keys -t $OLLAMA_PANE C-m C-m
```

## Troubleshooting

### Ollama Not Responding
```bash
# Check if Ollama server is running
ps aux | grep ollama

# Start Ollama service if needed
ollama serve &
```

### Model Too Slow
```bash
# Use smaller/faster model
ollama run codellama:7b  # Instead of :13b or :34b
```

### Out of Memory
```bash
# Stop Ollama
pkill ollama

# Restart with smaller model
ollama run mistral:7b  # Lighter than codellama
```

## Example: Complete Code Review Workflow

```bash
#!/bin/bash
# Use Ollama to review code, then report findings

SESSION=$(tmux display-message -p '#{session_name}')
FILE="src/auth.ts"

# Quick review with Ollama
echo "Reviewing $FILE with Ollama..."
REVIEW=$(ollama run --no-stream codellama "Review this code for:
1. Security issues
2. Type safety problems
3. Error handling gaps

$(cat $FILE)

Format as numbered list.")

# Save review
echo "$REVIEW" > /tmp/ollama-review.txt

# Report to orchestrator
./colony_message.sh send orchestrator "Completed Ollama code review of $FILE. Found:
$REVIEW"

echo "Review complete and sent to orchestrator"
```
