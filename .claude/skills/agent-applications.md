---
name: agent-applications
description: Building agent-based applications where agents are first-class application components. Use when the repository IS an agent application with workflows, state, and autonomous behavior.
---

# Agent Application Development Skill

## Overview

This skill helps you develop **agent applications** - systems where autonomous agents are the primary application logic. Unlike traditional applications where code is static and executed on demand, agent applications feature autonomous agents that perceive, reason, and act to achieve goals.

## What is an Agent Application?

An agent application is a repository where:
- **Agents are the application** - Not just tools, but core functionality
- **Agents have persistence** - State, memory, goals
- **Agents coordinate** - Work together toward shared objectives
- **Agents adapt** - Learn and improve over time

### Agent Application vs. Traditional Application

| Aspect | Agent Application | Traditional Application |
|--------|------------------|------------------------|
| Core logic | Autonomous agents | Functions/classes |
| Execution model | Goal-driven | Request-response |
| State management | Agent memory + DB | Database only |
| Adaptation | Learns from experience | Static logic |
| Coordination | Agent messaging | API calls |
| Development | Prompt eng + code | Code only |

## Application Architecture Patterns

### Pattern 1: Agent-as-Service

Agents provide services that other agents or users consume:

```
┌─────────────┐         ┌──────────────┐         ┌─────────────┐
│   User/API  │────────▶│ Router Agent │────────▶│Service Agent│
└─────────────┘         └──────────────┘         └─────────────┘
                               │                        │
                               ▼                        ▼
                        ┌──────────────┐         ┌─────────────┐
                        │ State Store  │         │   Tools     │
                        └──────────────┘         └─────────────┘
```

**Example**: Customer support system where agents handle inquiries

### Pattern 2: Multi-Agent Workflow

Agents pass work through a pipeline:

```
Input ──▶ Agent A ──▶ Agent B ──▶ Agent C ──▶ Output
            │           │           │
            ▼           ▼           ▼
         [State]    [State]    [State]
```

**Example**: Content pipeline (research → writing → editing → publishing)

### Pattern 3: Collaborative Problem Solving

Agents work together on complex tasks:

```
                    ┌─────────────┐
                    │Coordinator  │
                    │   Agent     │
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │Specialist│      │Specialist│      │Specialist│
    │ Agent A  │      │ Agent B  │      │ Agent C  │
    └─────────┘      └─────────┘      └─────────┘
```

**Example**: Software development (architect + developer + tester)

### Pattern 4: Reactive Agent System

Agents respond to events and triggers:

```
Events ──▶ Event Queue ──▶ Agent Pool ──▶ Actions
              │                │
              ▼                ▼
         [Rules/Triggers]  [Context]
```

**Example**: Monitoring system that responds to alerts

## Repository Structure

### Recommended Directory Structure

```
agent-application/
├── README.md                        # Application overview
├── agents/                          # Agent definitions
│   ├── coordinator/
│   │   ├── prompt.md               # Agent's system prompt
│   │   ├── config.yaml             # Agent configuration
│   │   └── tools.py                # Agent-specific tools
│   ├── researcher/
│   └── analyst/
├── workflows/                       # Workflow definitions
│   ├── customer-support.yaml
│   ├── content-pipeline.yaml
│   └── data-analysis.yaml
├── state/                          # Application state
│   ├── agent-memory/               # Per-agent memory
│   ├── shared-context/             # Shared state
│   └── task-queue/                 # Task management
├── tools/                          # Shared tools for agents
│   ├── database.py
│   ├── api-clients.py
│   └── notifications.py
├── config/                         # Application configuration
│   ├── agents.yaml                 # Agent definitions
│   ├── workflows.yaml              # Workflow configs
│   └── environment.yaml            # Environment settings
├── src/                            # Core application code
│   ├── runtime/                    # Agent runtime
│   ├── messaging/                  # Inter-agent communication
│   └── persistence/                # State management
├── tests/                          # Tests
│   ├── agent-tests/
│   ├── workflow-tests/
│   └── integration-tests/
└── docs/                           # Documentation
    ├── architecture.md
    ├── agent-guide.md
    └── api.md
```

## Core Components

### 1. Agent Definitions

Each agent needs a clear definition:

**agents/coordinator/config.yaml**:
```yaml
agent:
  id: coordinator
  name: "Workflow Coordinator"
  model: claude-opus-4-20250514

  # Agent's role and capabilities
  role: >
    Coordinate complex workflows by delegating tasks to specialist agents
    and synthesizing their results.

  # Available tools
  tools:
    - task_delegation
    - result_aggregation
    - workflow_management

  # Memory configuration
  memory:
    type: long-term
    max_tokens: 100000
    persistence: database

  # Behavioral parameters
  parameters:
    temperature: 0.7
    max_iterations: 10
    timeout_seconds: 300
```

**agents/coordinator/prompt.md**:
```markdown
# Workflow Coordinator Agent

You are a workflow coordinator responsible for managing complex multi-agent tasks.

## Your Capabilities
- Analyze incoming requests and decompose into subtasks
- Assign tasks to appropriate specialist agents
- Monitor task progress
- Handle failures and retries
- Synthesize results into final deliverables

## Workflow
1. Receive task request
2. Break down into subtasks
3. Identify required specialist agents
4. Delegate with clear instructions
5. Monitor progress
6. Handle any issues
7. Aggregate results
8. Deliver final output

## Available Agents
- researcher: Information gathering and analysis
- writer: Content creation
- reviewer: Quality assurance

## Tools
- delegate_task(agent_id, task, context)
- check_status(task_id)
- get_result(task_id)
- synthesize_results(results[])
```

### 2. State Management

Agents need persistent state:

```python
# src/persistence/agent_memory.py

class AgentMemory:
    def __init__(self, agent_id: str, storage_backend):
        self.agent_id = agent_id
        self.storage = storage_backend

    def save_context(self, key: str, value: any):
        """Save context for later retrieval"""
        self.storage.set(f"{self.agent_id}:{key}", value)

    def load_context(self, key: str) -> any:
        """Load saved context"""
        return self.storage.get(f"{self.agent_id}:{key}")

    def append_memory(self, memory_entry: dict):
        """Add to agent's long-term memory"""
        memories = self.storage.get_list(f"{self.agent_id}:memories")
        memories.append(memory_entry)
        self.storage.set_list(f"{self.agent_id}:memories", memories)

    def search_memory(self, query: str) -> list:
        """Search through memories"""
        memories = self.storage.get_list(f"{self.agent_id}:memories")
        # Implement semantic search or keyword matching
        return [m for m in memories if query.lower() in str(m).lower()]
```

### 3. Inter-Agent Communication

```python
# src/messaging/agent_bus.py

class AgentMessageBus:
    def __init__(self):
        self.subscribers = {}
        self.message_queue = []

    def publish(self, topic: str, message: dict, sender: str):
        """Publish message to topic"""
        msg = {
            "topic": topic,
            "sender": sender,
            "timestamp": datetime.now(),
            "payload": message
        }

        # Deliver to subscribers
        for agent_id in self.subscribers.get(topic, []):
            self.deliver(agent_id, msg)

    def subscribe(self, agent_id: str, topic: str):
        """Subscribe agent to topic"""
        if topic not in self.subscribers:
            self.subscribers[topic] = []
        self.subscribers[topic].append(agent_id)

    def send_direct(self, from_agent: str, to_agent: str, message: dict):
        """Send direct message to specific agent"""
        msg = {
            "type": "direct",
            "from": from_agent,
            "to": to_agent,
            "timestamp": datetime.now(),
            "payload": message
        }
        self.deliver(to_agent, msg)
```

### 4. Workflow Engine

```python
# src/runtime/workflow.py

class Workflow:
    def __init__(self, config_path: str):
        self.config = load_workflow_config(config_path)
        self.state = WorkflowState()

    async def execute(self, input_data: dict) -> dict:
        """Execute workflow from start to finish"""
        current_step = self.config.start_step

        while current_step:
            step_config = self.config.steps[current_step]

            # Execute step
            result = await self.execute_step(step_config, input_data)

            # Update state
            self.state.record_step(current_step, result)

            # Determine next step
            current_step = self.determine_next_step(
                current_step, result, step_config
            )

        return self.state.get_final_output()

    async def execute_step(self, step_config, input_data):
        """Execute a single workflow step"""
        if step_config.type == "agent_task":
            agent = self.get_agent(step_config.agent_id)
            return await agent.process(input_data, step_config.instructions)

        elif step_config.type == "parallel":
            tasks = [
                self.execute_step(sub_step, input_data)
                for sub_step in step_config.parallel_steps
            ]
            return await asyncio.gather(*tasks)

        elif step_config.type == "conditional":
            condition_result = evaluate_condition(
                step_config.condition, input_data
            )
            branch = step_config.if_true if condition_result else step_config.if_false
            return await self.execute_step(branch, input_data)
```

## Development Workflows

### Creating a New Agent

```bash
# 1. Create agent directory structure
mkdir -p agents/new-agent
cd agents/new-agent

# 2. Create agent configuration
cat > config.yaml <<EOF
agent:
  id: new-agent
  name: "New Agent Name"
  model: claude-sonnet-4-20250514
  role: "Agent's role and responsibilities"
  tools:
    - tool1
    - tool2
  memory:
    type: long-term
    max_tokens: 50000
EOF

# 3. Create agent prompt
cat > prompt.md <<EOF
# New Agent

You are [role description].

## Capabilities
- Capability 1
- Capability 2

## Workflow
1. Step 1
2. Step 2

## Tools
- tool1: Description
- tool2: Description
EOF

# 4. Create agent-specific tools (if needed)
cat > tools.py <<EOF
def custom_tool(param: str) -> str:
    """Tool description"""
    # Implementation
    pass
EOF

# 5. Register agent in config/agents.yaml
# 6. Write tests
# 7. Integrate into workflows
```

### Defining a Workflow

**workflows/customer-support.yaml**:
```yaml
workflow:
  id: customer-support
  name: "Customer Support Workflow"
  description: "Handle customer inquiries with intelligent routing"

  start_step: classify

  steps:
    classify:
      type: agent_task
      agent_id: classifier
      instructions: "Classify customer inquiry into category"
      next:
        - condition: "category == 'technical'"
          step: technical_support
        - condition: "category == 'billing'"
          step: billing_support
        - default: general_support

    technical_support:
      type: agent_task
      agent_id: technical-support
      instructions: "Provide technical assistance"
      next: quality_check

    billing_support:
      type: agent_task
      agent_id: billing-support
      instructions: "Handle billing inquiry"
      next: quality_check

    general_support:
      type: agent_task
      agent_id: general-support
      instructions: "Provide general assistance"
      next: quality_check

    quality_check:
      type: agent_task
      agent_id: qa-agent
      instructions: "Review response quality"
      next:
        - condition: "quality_score > 0.8"
          step: send_response
        - default: escalate

    escalate:
      type: agent_task
      agent_id: supervisor
      instructions: "Handle escalation"
      next: send_response

    send_response:
      type: action
      action: send_to_customer
      next: null  # End workflow
```

### Testing Agents

```python
# tests/agent-tests/test_coordinator.py

import pytest
from src.runtime.agent import Agent
from src.persistence.memory import InMemoryStorage

@pytest.fixture
def coordinator_agent():
    """Create coordinator agent for testing"""
    config = load_agent_config("agents/coordinator/config.yaml")
    memory = AgentMemory("coordinator-test", InMemoryStorage())
    return Agent(config, memory)

def test_task_delegation(coordinator_agent):
    """Test that coordinator properly delegates tasks"""
    request = {
        "task": "Analyze market trends",
        "requirements": ["data collection", "analysis", "report"]
    }

    result = coordinator_agent.process(request)

    assert len(result.delegated_tasks) == 3
    assert "researcher" in result.assigned_agents
    assert result.status == "delegated"

def test_result_aggregation(coordinator_agent):
    """Test aggregation of specialist results"""
    sub_results = [
        {"agent": "researcher", "data": {...}},
        {"agent": "analyst", "insights": {...}},
        {"agent": "writer", "report": "..."}
    ]

    final_result = coordinator_agent.aggregate_results(sub_results)

    assert final_result.contains_all_sections()
    assert final_result.is_coherent()
```

## Colony Integration

### Agent Application Colony Configuration

```yaml
# colony.yml for agent application

repository:
  repo_type: application
  purpose: "Multi-agent customer support system"
  context: |
    This is an agent application where autonomous agents handle customer inquiries.
    Agents coordinate through message passing and maintain state in a shared database.

agents:
  # Runtime manager
  - id: runtime-manager
    role: Agent Runtime Manager
    focus: Manage agent lifecycle and runtime
    instructions: |
      Responsibilities:
      - Monitor agent health
      - Restart failed agents
      - Scale agent pools based on load
      - Manage resource allocation
      - Log agent activities

  # Application developer
  - id: app-developer
    role: Application Developer
    focus: Develop agent logic and workflows
    instructions: |
      Development tasks:
      - Create new agent definitions
      - Implement custom tools
      - Define workflows
      - Write tests
      - Update documentation

  # Prompt engineer
  - id: prompt-engineer
    role: Prompt Engineer
    focus: Optimize agent prompts and behaviors
    instructions: |
      Prompt optimization:
      - Review agent performance
      - Refine system prompts
      - Test prompt variations
      - Document best prompts
      - Measure improvements

  # Integration tester
  - id: integration-tester
    role: Integration Tester
    focus: Test agent interactions and workflows
    instructions: |
      Testing focus:
      - Test agent coordination
      - Verify workflow execution
      - Check state persistence
      - Validate error handling
      - Load testing
```

### Monitoring Agent Applications

```python
# tools/monitoring.py

class AgentMonitor:
    def __init__(self, agent_id: str):
        self.agent_id = agent_id
        self.metrics = MetricsCollector()

    def record_task_start(self, task_id: str):
        """Record when agent starts a task"""
        self.metrics.record({
            "agent": self.agent_id,
            "event": "task_start",
            "task_id": task_id,
            "timestamp": datetime.now()
        })

    def record_task_complete(self, task_id: str, duration: float):
        """Record task completion"""
        self.metrics.record({
            "agent": self.agent_id,
            "event": "task_complete",
            "task_id": task_id,
            "duration_seconds": duration,
            "timestamp": datetime.now()
        })

    def record_error(self, error: Exception, context: dict):
        """Record agent error"""
        self.metrics.record({
            "agent": self.agent_id,
            "event": "error",
            "error_type": type(error).__name__,
            "error_message": str(error),
            "context": context,
            "timestamp": datetime.now()
        })

    def get_health_status(self) -> dict:
        """Get agent health metrics"""
        return {
            "agent_id": self.agent_id,
            "status": self.calculate_status(),
            "tasks_completed_1h": self.count_tasks_last_hour(),
            "average_task_duration": self.avg_task_duration(),
            "error_rate": self.calculate_error_rate(),
            "last_active": self.get_last_activity_time()
        }
```

## Best Practices

### Agent Design

#### DO:
- ✅ Give each agent a clear, focused role
- ✅ Define explicit capabilities and tools
- ✅ Provide examples in prompts
- ✅ Implement proper error handling
- ✅ Log agent decisions and actions
- ✅ Test agents in isolation and integration
- ✅ Version control agent prompts
- ✅ Monitor agent performance

#### DON'T:
- ❌ Create generic "do everything" agents
- ❌ Hard-code business logic in prompts
- ❌ Ignore agent errors or failures
- ❌ Skip testing agent coordination
- ❌ Forget to handle edge cases
- ❌ Let agents run unbounded
- ❌ Store sensitive data in prompts

### State Management

#### DO:
- ✅ Persist important state to database
- ✅ Version state schemas
- ✅ Implement state recovery
- ✅ Clean up old state periodically
- ✅ Encrypt sensitive state data
- ✅ Back up state regularly

#### DON'T:
- ❌ Store everything in memory
- ❌ Share mutable state between agents
- ❌ Forget to handle state migration
- ❌ Leave sensitive data unencrypted
- ❌ Let state grow unbounded

### Workflow Design

#### DO:
- ✅ Define clear workflow stages
- ✅ Handle failures gracefully
- ✅ Implement retry logic
- ✅ Set reasonable timeouts
- ✅ Monitor workflow progress
- ✅ Log workflow execution
- ✅ Support workflow rollback

#### DON'T:
- ❌ Create circular dependencies
- ❌ Ignore timeouts
- ❌ Skip error handling
- ❌ Make workflows too complex
- ❌ Hard-code workflow logic

## Common Patterns

### Pattern: Request-Response Agent

```python
class RequestResponseAgent:
    async def handle_request(self, request: dict) -> dict:
        # 1. Validate request
        self.validate(request)

        # 2. Load context
        context = self.memory.load_context(request.id)

        # 3. Process with LLM
        response = await self.llm.complete(
            prompt=self.build_prompt(request, context),
            tools=self.tools
        )

        # 4. Save context
        self.memory.save_context(request.id, response.context)

        # 5. Return response
        return self.format_response(response)
```

### Pattern: Supervisor-Worker

```python
class SupervisorAgent:
    def __init__(self, worker_pool):
        self.workers = worker_pool
        self.task_queue = TaskQueue()

    async def delegate_task(self, task: dict):
        # Select best worker for task
        worker = self.select_worker(task)

        # Assign task
        await worker.assign(task)

        # Monitor progress
        self.task_queue.add_monitored_task(task.id, worker.id)

    async def monitor_workers(self):
        while True:
            for worker in self.workers:
                if worker.needs_help():
                    await self.assist_worker(worker)

                if worker.is_stuck():
                    await self.reassign_task(worker.current_task)

            await asyncio.sleep(10)
```

### Pattern: Event-Driven Agent

```python
class EventDrivenAgent:
    def __init__(self, event_bus):
        self.event_bus = event_bus
        self.handlers = {}

    def on(self, event_type: str, handler: callable):
        """Register event handler"""
        self.handlers[event_type] = handler

    async def start(self):
        """Start listening for events"""
        async for event in self.event_bus.subscribe(self.agent_id):
            await self.handle_event(event)

    async def handle_event(self, event: dict):
        """Handle incoming event"""
        event_type = event["type"]

        if event_type in self.handlers:
            await self.handlers[event_type](event)
        else:
            logger.warning(f"No handler for event type: {event_type}")
```

## Deployment

### Production Considerations

1. **Scalability**
   - Horizontal scaling of agent pools
   - Load balancing across agents
   - Queue management for tasks

2. **Reliability**
   - Health checks for agents
   - Automatic restart on failures
   - State persistence and recovery
   - Graceful degradation

3. **Security**
   - Agent authentication
   - Tool access control
   - Input validation
   - Output sanitization
   - Audit logging

4. **Observability**
   - Agent activity logging
   - Performance metrics
   - Error tracking
   - Workflow visualization
   - Cost monitoring (LLM calls)

### Example Deployment Configuration

```yaml
# config/production.yaml

environment: production

agents:
  coordinator:
    replicas: 2
    model: claude-opus-4-20250514
    max_concurrent_tasks: 10

  worker:
    replicas: 5
    model: claude-sonnet-4-20250514
    max_concurrent_tasks: 5
    autoscale:
      min_replicas: 3
      max_replicas: 20
      target_queue_length: 10

database:
  type: postgresql
  host: db.example.com
  pool_size: 20

messaging:
  type: redis
  host: cache.example.com

monitoring:
  metrics_port: 9090
  health_check_interval: 30
  log_level: info
```

## Troubleshooting

### Agent Not Responding

1. Check agent health status
2. Review recent error logs
3. Verify tool availability
4. Check memory/resource usage
5. Restart agent if necessary

### Workflow Stuck

1. Check current workflow state
2. Identify stuck step
3. Review step logs
4. Check for deadlocks
5. Manually advance or restart workflow

### High Error Rate

1. Analyze error patterns
2. Check tool failures
3. Review prompt effectiveness
4. Validate input data
5. Adjust retry logic

---

## Quick Start

```bash
# 1. Clone agent application template
git clone https://github.com/your-org/agent-app-template

# 2. Install dependencies
pip install -r requirements.txt

# 3. Configure agents
cp config/agents.example.yaml config/agents.yaml
# Edit config/agents.yaml

# 4. Start runtime
python -m src.runtime.server

# 5. Deploy agents (in colony)
colony start
```

Agent applications represent the future of software - systems that can perceive, reason, and act autonomously to achieve goals. Build thoughtfully!
