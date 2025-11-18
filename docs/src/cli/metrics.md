# colony metrics

View and manage metrics for monitoring colony performance.

## Synopsis

```bash
colony metrics <SUBCOMMAND>
```

## Description

The metrics system collects and displays performance data about your colony, including agent activity, task completion, workflow execution, and system resources.

## Subcommands

### list

List all registered metrics with current values.

```bash
colony metrics list
```

**Output**:
```
Available Metrics

Agent Metrics
  agent.tasks.completed [counter] - 15.00 tasks
    Number of tasks completed by agents
  agent.tasks.failed [counter] - 2.00 tasks
    Number of tasks failed by agents

Task Metrics
  task.queue.depth [gauge] - 5.00 tasks
    Current number of tasks in the queue

Workflow Metrics
  workflow.runs.completed [counter] - 8.00 runs
    Number of workflow runs completed

System Metrics
  system.memory.used [gauge] - 1024.50 MB
    Memory used by the colony system
```

### show

Show detailed statistics for a specific metric.

```bash
colony metrics show <NAME> [--hours HOURS]
```

**Arguments**:
- `NAME` - Metric name (e.g., `agent.tasks.completed`)

**Options**:
- `--hours` - Time period in hours (default: 1)

**Example**:
```bash
colony metrics show agent.tasks.completed --hours 24
```

**Output**:
```
Metric: agent.tasks.completed
Type: Counter
Description: Number of tasks completed by agents
Unit: tasks

Statistics (last 24 hours)

  Current: 45.00 tasks
  Average: 38.50 tasks
  Min: 15.00 tasks
  Max: 45.00 tasks
  Total: 462.00 tasks
  Data Points: 12

Recent Values

  2024-01-15 14:30:00 - 45.00 tasks
  2024-01-15 13:30:00 - 42.00 tasks
  2024-01-15 12:30:00 - 38.00 tasks
```

### export

Export all metrics to JSON format.

```bash
colony metrics export [-o FILE]
```

**Options**:
- `-o, --output` - Output file path (default: stdout)

**Example**:
```bash
# Export to file
colony metrics export -o metrics.json

# Export to stdout
colony metrics export | jq '.[] | select(.name | contains("agent"))'
```

**JSON Format**:
```json
[
  {
    "name": "agent.tasks.completed",
    "metric_type": "Counter",
    "description": "Number of tasks completed by agents",
    "unit": "tasks",
    "points": [
      {
        "timestamp": "2024-01-15T14:30:00Z",
        "value": 45.0,
        "labels": {}
      }
    ]
  }
]
```

### clear

Clear old metrics data or all metrics.

```bash
colony metrics clear [--all]
```

**Options**:
- `--all` - Clear all metrics, not just old data

**Examples**:
```bash
# Clear data older than retention period (24 hours)
colony metrics clear

# Clear all metrics
colony metrics clear --all
```

### init

Initialize sample metrics for testing.

```bash
colony metrics init
```

Creates standard metrics:
- agent.tasks.completed
- agent.tasks.failed
- task.queue.depth
- task.execution_time
- workflow.runs.completed
- system.memory.used

### record

Record a sample metric value (for testing).

```bash
colony metrics record <NAME> <VALUE>
```

**Arguments**:
- `NAME` - Metric name
- `VALUE` - Numeric value to record

**Example**:
```bash
colony metrics record agent.tasks.completed 10
colony metrics record system.cpu.percent 45.5
```

## Metric Types

### Counter

Monotonically increasing values:
- Task completions
- Workflow runs
- Errors

**Use case**: Counting events that only go up.

### Gauge

Point-in-time values that can go up or down:
- Queue depth
- Memory usage
- Active agents

**Use case**: Current state snapshots.

### Histogram

Distribution of values over time:
- Execution times
- Response latencies
- Resource usage patterns

**Use case**: Understanding value distributions.

## Standard Metrics

### Agent Metrics

- `agent.tasks.completed` - Tasks completed by agents
- `agent.tasks.failed` - Tasks that failed
- `agent.active_time` - Time agent spent active
- `agent.idle_time` - Time agent spent idle

### Task Metrics

- `task.queue.depth` - Current tasks in queue
- `task.wait_time` - Time tasks wait before starting
- `task.execution_time` - Time to complete tasks

### Workflow Metrics

- `workflow.runs.started` - Workflows started
- `workflow.runs.completed` - Workflows completed
- `workflow.runs.failed` - Workflows that failed
- `workflow.step.duration` - Time per workflow step

### System Metrics

- `system.memory.used` - Memory usage
- `system.cpu.percent` - CPU utilization

## Integration

### Export to Monitoring Tools

```bash
# Prometheus format (future)
colony metrics export --format prometheus

# Grafana (use JSON export)
colony metrics export | curl -X POST grafana-api/metrics

# Custom processing
colony metrics export | jq '...'
```

### Automated Monitoring

```bash
# Continuous monitoring
while true; do
  colony metrics export -o metrics-$(date +%s).json
  sleep 60
done
```

### Alerting

```bash
# Check thresholds
QUEUE_DEPTH=$(colony metrics show task.queue.depth --json | jq '.current')
if [ $QUEUE_DEPTH -gt 100 ]; then
  echo "Alert: Queue depth is $QUEUE_DEPTH"
fi
```

## Examples

### Monitor Task Completion Rate

```bash
# Show last 24 hours
colony metrics show agent.tasks.completed --hours 24

# Compare with failures
colony metrics show agent.tasks.failed --hours 24
```

### Track System Resources

```bash
# Memory usage
colony metrics show system.memory.used

# Record custom metric
colony metrics record system.cpu.percent $(top -bn1 | grep "Cpu(s)" | awk '{print $2}')
```

### Generate Daily Reports

```bash
#!/bin/bash
# daily-metrics.sh
DATE=$(date +%Y-%m-%d)
colony metrics export -o reports/metrics-$DATE.json

# Email report
mail -s "Colony Metrics $DATE" admin@example.com < reports/metrics-$DATE.json
```

## Retention

- **Default**: 24 hours
- **Configurable**: Future feature
- **Pruning**: Automatic via `colony metrics clear`

## See Also

- [Workflows](./workflow.md) - Monitor workflow execution
- [Status](./status.md) - Agent status
- [TUI](./tui.md) - Real-time visualization
