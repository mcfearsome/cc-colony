"""
Template: Multi-Tool Composition
Purpose: Template for calling multiple MCP tools in sequence or parallel
Use Case: When you need to compose multiple tool operations together

How to Adapt:
1. Define your tool calls in the tools list
2. Choose sequential or parallel execution pattern
3. Add logic to process and combine results
4. Customize error handling strategy (fail-fast vs. partial success)
5. Update return value with your aggregated data

Example Usage:
Use for workflows like: fetch data → transform → store results

Execution:
python your-script.py
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from lib.mcp_client import call_mcp_tool, call_mcp_tools_parallel

import asyncio
from typing import List, Dict, Any

# TODO: Define your tool calls
tools = [
    {
        'name': 'mcp__server1__tool1',
        'params': {  # tool 1 parameters
        }
    },
    {
        'name': 'mcp__server2__tool2',
        'params': {  # tool 2 parameters
        }
    },
    {
        'name': 'mcp__server3__tool3',
        'params': {  # tool 3 parameters
        }
    }
]

print(f'Executing {len(tools)} tool operations...')

try:
    # OPTION A: Sequential Execution (one after another)
    # Use when results depend on each other
    """
    results = []
    for tool in tools:
        print(f"Calling {tool['name']}...")
        result = await call_mcp_tool(tool['name'], tool['params'])
        results.append(result)

        # TODO: Add logic between calls if needed
        # Example: use result from previous call in next call
    """

    # OPTION B: Parallel Execution (all at once)
    # Use when operations are independent
    async def call_tool(tool: Dict[str, Any]) -> Any:
        print(f"Calling {tool['name']}...")
        return await call_mcp_tool(tool['name'], tool['params'])

    results = await asyncio.gather(*[call_tool(tool) for tool in tools])

    print('All tool calls completed')

    # TODO: Process and combine results
    combined = {
        # Example: Combine all results
        'results': results,
        # Add your custom aggregation logic
    }

    # TODO: Return aggregated data
    return {
        'success': True,
        'tools_called': len(tools),
        'data': combined
    }

except Exception as error:
    print(f'Error in multi-tool execution: {error}')

    # TODO: Decide error handling strategy
    # - Fail fast: raise error to stop everything
    # - Partial success: return what succeeded so far

    return {
        'success': False,
        'error': str(error)
    }

# OPTION C: Parallel with Partial Failure Handling
# Use when you want to continue even if some calls fail
"""
results = await asyncio.gather(*[
    call_tool(tool) for tool in tools
], return_exceptions=True)

successes = [r for r in results if not isinstance(r, Exception)]
failures = [r for r in results if isinstance(r, Exception)]

print(f'Success: {len(successes)}, Failed: {len(failures)}')

return {
    'success': len(failures) == 0,
    'success_count': len(successes),
    'failure_count': len(failures),
    'data': successes,
    'errors': [str(f) for f in failures]
}
"""
