"""
Script: Parallel Execution
Purpose: Execute multiple independent tool calls concurrently for better performance
Use Case: When you have multiple operations that don't depend on each other
MCP Tools Used: Multiple independent MCP tools (databases, APIs, services)

How to Adapt:
1. Replace example tool calls with your actual independent operations
2. Adjust the number of parallel operations
3. Decide on error handling strategy (fail-fast vs. partial success)
4. Customize result aggregation logic
5. Add timeout handling if needed

Example Usage:
Fetch data from multiple sources simultaneously, aggregate analytics from
different services, or perform bulk operations in parallel
"""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../.."))
from lib.mcp_client import (
    call_mcp_tool,
)

import asyncio
from typing import Dict, Any
from datetime import datetime


async def parallel_execution() -> Dict[str, Any]:
    """Execute multiple operations in parallel with graceful error handling."""
    print("=== Starting Parallel Execution ===")
    start_time = datetime.now()

    # Define independent operations to run in parallel
    operations = [
        {
            "name": "Fetch Users",
            "tool": "mcp__database__query",
            "params": {"table": "users", "limit": 1000},
        },
        {
            "name": "Fetch Orders",
            "tool": "mcp__database__query",
            "params": {"table": "orders", "limit": 1000},
        },
        {
            "name": "Fetch Products",
            "tool": "mcp__database__query",
            "params": {"table": "products", "limit": 500},
        },
        {
            "name": "Get Analytics",
            "tool": "mcp__analytics__get_metrics",
            "params": {"period": "last_7_days"},
        },
        {
            "name": "Load Config",
            "tool": "mcp__config__get",
            "params": {"key": "app_settings"},
        },
    ]

    print(f"Executing {len(operations)} operations in parallel...")

    # Execute all operations concurrently with gather and return_exceptions
    # This allows us to handle partial failures gracefully
    async def execute_operation(op: Dict[str, Any], index: int) -> Dict[str, Any]:
        op_start = datetime.now()
        print(f"[{index + 1}/{len(operations)}] Starting: {op['name']}")

        try:
            result = await call_mcp_tool(op["tool"], op["params"])
            duration = (datetime.now() - op_start).total_seconds() * 1000
            print(
                f"✓ [{index + 1}/{len(operations)}] {op['name']} completed ({duration:.0f}ms)"
            )

            return {
                "name": op["name"],
                "success": True,
                "data": result,
                "duration": duration,
            }
        except Exception as error:
            duration = (datetime.now() - op_start).total_seconds() * 1000
            print(
                f"✗ [{index + 1}/{len(operations)}] {op['name']} failed ({duration:.0f}ms): {error}"
            )

            return {
                "name": op["name"],
                "success": False,
                "error": str(error),
                "duration": duration,
            }

    # Execute all operations
    results = await asyncio.gather(
        *[execute_operation(op, i) for i, op in enumerate(operations)],
        return_exceptions=True,
    )

    # Analyze results
    successful = [r for r in results if not isinstance(r, Exception) and r["success"]]
    failed = [r for r in results if isinstance(r, Exception) or not r["success"]]

    errors = []
    for r in failed:
        if isinstance(r, Exception):
            errors.append(str(r))
        else:
            errors.append(r.get("error", "Unknown error"))

    total_duration = (datetime.now() - start_time).total_seconds() * 1000

    # Calculate estimated sequential time (sum of individual durations)
    estimated_sequential = sum(r["duration"] for r in successful)
    speedup = (
        f"{estimated_sequential / total_duration:.2f}x faster"
        if estimated_sequential > 0
        else "N/A"
    )

    print("\n=== Execution Summary ===")
    print(f"Total time: {total_duration:.0f}ms")
    print(f"Estimated sequential time: {estimated_sequential:.0f}ms")
    print(f"Speedup: {speedup}")
    print(f"Completed: {len(successful)}/{len(operations)}")
    print(f"Failed: {len(failed)}/{len(operations)}")

    if failed:
        print("\nFailed operations:")
        for i, err in enumerate(errors, 1):
            print(f"  {i}. {err}")

    return {
        "success": len(failed) == 0,
        "completed": len(successful),
        "failed": len(failed),
        "results": successful,
        "errors": errors,
        "duration": total_duration,
        "speedup": speedup,
    }


# Alternative: Fail-fast approach using gather without return_exceptions
# Uncomment to use this pattern instead
"""
async def parallel_execution_fail_fast():
    print('=== Parallel Execution (Fail-Fast) ===')

    try:
        users, orders, products, analytics, config = await asyncio.gather(
            call_mcp_tool('mcp__database__query', {'table': 'users'}),
            call_mcp_tool('mcp__database__query', {'table': 'orders'}),
            call_mcp_tool('mcp__database__query', {'table': 'products'}),
            call_mcp_tool('mcp__analytics__get_metrics', {'period': 'last_7_days'}),
            call_mcp_tool('mcp__config__get', {'key': 'app_settings'})
        )

        print('All operations completed successfully')

        return {
            'success': True,
            'data': {
                'users': users,
                'orders': orders,
                'products': products,
                'analytics': analytics,
                'config': config
            }
        }
    except Exception as error:
        print(f'At least one operation failed: {error}')
        # If any operation fails, all results are lost
        raise
"""

# Execute
result = await parallel_execution()

if result["success"]:
    print("\n✓ All operations completed successfully")
else:
    print(f"\n⚠ Completed with {result['failed']} failures")
