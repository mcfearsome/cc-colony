# Custom Templates

Advanced guide for creating custom agent templates.

## Template Schema

```yaml
name: string          # Template name
version: string       # Semantic version
author: string        # Author name
description: string   # Description
license: string       # License (MIT, Apache-2.0, etc.)

agent:
  role: string
  focus: string
  model: string
  startup_prompt: string
  mcp_servers: object

requirements:
  repo_types: array
```

See [Templates](../concepts/templates.md) for usage examples.
