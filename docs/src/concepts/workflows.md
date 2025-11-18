# Workflows

Workflows define complex multi-agent processes.

## Overview

Workflows are DAG-based sequences of steps with:
- Dependencies between steps
- Retry policies
- Error handling
- Input/output passing

## Workflow Definition

```yaml
name: code-review-pipeline
description: Automated code review workflow

steps:
  - id: run-tests
    agent: test-engineer
    action: run_tests

  - id: security-scan
    agent: security-auditor
    action: security_scan
    depends_on: [run-tests]

  - id: code-review
    agent: code-reviewer
    action: review
    depends_on: [run-tests, security-scan]
```

See [Workflow CLI](../cli/workflow.md) for commands.
