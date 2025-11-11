"""
Script: Error Recovery
Purpose: Demonstrate retry logic, exponential backoff, and fallback strategies
Use Case: Building resilient systems that handle failures gracefully
MCP Tools Used: Primary and fallback services, cache systems

How to Adapt:
1. Replace primary/secondary/cache tool names with your actual tools
2. Adjust retry counts and backoff timing for your use case
3. Customize fallback strategy (secondary service, cache, defaults)
4. Add circuit breaker pattern if needed
5. Implement custom error classification (retryable vs. permanent)

Example Usage:
Fetching data from unreliable APIs, database queries with failover,
or any operation that might need multiple attempts
"""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../.."))
from lib.mcp_client import (
    call_mcp_tool,
)

import asyncio
import json
from typing import Dict, Any, TypedDict
from datetime import datetime


class RetryConfig(TypedDict):
    max_retries: int
    initial_delay: float  # seconds
    max_delay: float  # seconds
    backoff_multiplier: float


class RecoveryResult(TypedDict):
    success: bool
    data: Any
    source: str
    attempts: int
    duration: float
    errors: list[str]


async def call_with_retry(
    tool_name: str, params: Dict[str, Any], config: RetryConfig
) -> Any:
    """Execute a tool call with exponential backoff retry logic."""
    last_error = None

    for attempt in range(config["max_retries"]):
        try:
            print(f"Attempt {attempt + 1}/{config['max_retries']}: {tool_name}")
            result = await call_mcp_tool(tool_name, params)
            print(f"✓ Success on attempt {attempt + 1}")
            return result
        except Exception as error:
            last_error = error
            print(f"✗ Attempt {attempt + 1} failed: {error}")

            # Don't wait after the last attempt
            if attempt < config["max_retries"] - 1:
                # Calculate delay with exponential backoff
                delay = min(
                    config["initial_delay"] * (config["backoff_multiplier"] ** attempt),
                    config["max_delay"],
                )

                print(f"Waiting {delay}s before retry...")
                await asyncio.sleep(delay)

    # All retries exhausted
    raise last_error


async def fetch_data_with_recovery(data_id: str) -> RecoveryResult:
    """Fetch data with comprehensive error recovery strategy."""
    start_time = datetime.now()
    errors: list[str] = []
    attempts = 0

    # Configuration for retry logic
    retry_config: RetryConfig = {
        "max_retries": 3,
        "initial_delay": 1.0,  # Start with 1 second
        "max_delay": 10.0,  # Cap at 10 seconds
        "backoff_multiplier": 2,  # Double delay each time (1s, 2s, 4s)
    }

    print("=== Starting Data Fetch with Error Recovery ===")
    print(f"Target: {data_id}")

    # Strategy 1: Try primary service with retries
    print("\n[Strategy 1] Attempting primary service with retries...")
    try:
        data = await call_with_retry(
            "mcp__primary__get_data", {"id": data_id, "timeout": 5000}, retry_config
        )

        attempts += retry_config["max_retries"]

        duration = (datetime.now() - start_time).total_seconds()
        return {
            "success": True,
            "data": data,
            "source": "primary",
            "attempts": attempts,
            "duration": duration,
            "errors": errors,
        }
    except Exception as error:
        errors.append(f"Primary service: {error}")
        print(f"✗ Primary service failed after {retry_config['max_retries']} attempts")

    # Strategy 2: Try secondary/fallback service
    print("\n[Strategy 2] Attempting secondary service...")
    try:
        attempts += 1
        data = await call_mcp_tool("mcp__secondary__get_data", {"id": data_id})

        print("✓ Secondary service succeeded")

        duration = (datetime.now() - start_time).total_seconds()
        return {
            "success": True,
            "data": data,
            "source": "secondary",
            "attempts": attempts,
            "duration": duration,
            "errors": errors,
        }
    except Exception as error:
        errors.append(f"Secondary service: {error}")
        print(f"✗ Secondary service failed: {error}")

    # Strategy 3: Try cache as last resort
    print("\n[Strategy 3] Attempting cached data...")
    try:
        attempts += 1
        cached = await call_mcp_tool("mcp__cache__get", {"key": f"data:{data_id}"})

        if cached and cached.get("value"):
            print("✓ Found cached data")
            print("⚠ Using potentially stale cached data")

            duration = (datetime.now() - start_time).total_seconds()
            return {
                "success": True,
                "data": cached["value"],
                "source": "cache",
                "attempts": attempts,
                "duration": duration,
                "errors": errors,
            }
        else:
            raise Exception("No cached data available")
    except Exception as error:
        errors.append(f"Cache: {error}")
        print(f"✗ Cache lookup failed: {error}")

    # Strategy 4: Return default/empty data
    print("\n[Strategy 4] All strategies exhausted, returning default data")
    print("⚠ Returning default empty result")

    duration = (datetime.now() - start_time).total_seconds()
    return {
        "success": False,
        "data": None,
        "source": "default",
        "attempts": attempts,
        "duration": duration,
        "errors": errors,
    }


async def batch_process_with_recovery(ids: list[str]) -> Dict[str, Any]:
    """Batch processing with error recovery."""
    print(f"\n=== Batch Processing {len(ids)} items ===\n")

    results = await asyncio.gather(
        *[fetch_data_with_recovery(item_id) for item_id in ids]
    )

    successful = [r for r in results if r["success"] and r["source"] != "default"]
    from_cache = [r for r in results if r["success"] and r["source"] == "cache"]
    failed = [r for r in results if not r["success"]]

    print("\n=== Batch Processing Summary ===")
    print(f"Total items: {len(ids)}")
    print(f"Successful: {len(successful)}")
    print(f"From cache: {len(from_cache)}")
    print(f"Failed: {len(failed)}")

    if from_cache:
        print(f"⚠ {len(from_cache)} items retrieved from potentially stale cache")

    return {"successful": successful, "from_cache": from_cache, "failed": failed}


# Execute single fetch
print("Example 1: Single Item Recovery")
single_result = await fetch_data_with_recovery("user-123")
result_summary = {
    "success": single_result["success"],
    "source": single_result["source"],
    "attempts": single_result["attempts"],
    "duration": f'{single_result["duration"]:.2f}s',
    "error_count": len(single_result["errors"]),
}
print(f"\nResult: {json.dumps(result_summary, indent=2)}")

# Execute batch processing
print("\n\n" + "=" * 50)
print("Example 2: Batch Processing with Recovery")
batch_result = await batch_process_with_recovery(["user-123", "user-456", "user-789"])
