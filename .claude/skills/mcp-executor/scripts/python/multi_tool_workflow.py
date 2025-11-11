"""
Script: Multi-Tool Workflow
Purpose: Demonstrates sequential operations with data flow between tools
Use Case: Building data processing pipelines where each step depends on the previous
MCP Tools Used: mcp__filesystem__*, mcp__database__*, example tools

How to Adapt:
1. Replace example tool names with your actual MCP tools
2. Modify the data transformation logic for your use case
3. Adjust error handling based on your requirements
4. Add or remove pipeline steps as needed
5. Customize the output format

Example Usage:
Processing workflow: Fetch → Transform → Validate → Store → Report
"""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../.."))
from lib.mcp_client import (
    call_mcp_tool,
)

import json
from typing import Dict, Any
from datetime import datetime


async def multi_tool_workflow() -> Dict[str, Any]:
    """Execute a multi-step workflow with data flow between tools."""
    start_time = datetime.now()
    total_steps = 5
    steps_completed = 0

    try:
        print("=== Starting Multi-Tool Workflow ===")

        # Step 1: Fetch data from source
        print("[Step 1/5] Fetching data from source...")
        raw_data = await call_mcp_tool(
            "mcp__database__query", {"table": "source_data", "limit": 100}
        )
        steps_completed += 1
        print(f"✓ Fetched {len(raw_data)} records")

        # Step 2: Transform data
        print("[Step 2/5] Transforming data...")
        transformed = [
            {
                "id": record["id"],
                "name": record.get("full_name") or record.get("name"),
                "email": record["email"].lower(),
                "created_at": datetime.fromisoformat(record["timestamp"]).isoformat(),
                "status": "active" if record.get("is_active") else "inactive",
            }
            for record in raw_data
        ]
        steps_completed += 1
        print(f"✓ Transformed {len(transformed)} records")

        # Step 3: Validate data
        print("[Step 3/5] Validating data...")
        validated = [
            record
            for record in transformed
            if record["id"]
            and record["name"]
            and record["email"]
            and "@" in record["email"]
        ]

        invalid_count = len(transformed) - len(validated)
        if invalid_count > 0:
            print(f"⚠ Filtered out {invalid_count} invalid records")
        steps_completed += 1
        print(f"✓ Validated {len(validated)} records")

        # Step 4: Store processed data
        print("[Step 4/5] Storing processed data...")
        store_result = await call_mcp_tool(
            "mcp__database__bulk_insert",
            {"table": "processed_data", "records": validated},
        )
        steps_completed += 1
        print(f"✓ Stored {store_result['inserted']} records")

        # Step 5: Generate report
        print("[Step 5/5] Generating report...")
        report = {
            "timestamp": datetime.now().isoformat(),
            "source_records": len(raw_data),
            "transformed": len(transformed),
            "validated": len(validated),
            "invalid": invalid_count,
            "stored": store_result["inserted"],
            "success_rate": f"{(len(validated) / len(raw_data) * 100):.2f}%",
        }

        await call_mcp_tool(
            "mcp__filesystem__write_file",
            {
                "path": "/tmp/workflow-report.json",
                "content": json.dumps(report, indent=2),
            },
        )
        steps_completed += 1
        print("✓ Report saved to /tmp/workflow-report.json")

        duration = (datetime.now() - start_time).total_seconds() * 1000
        print(f"=== Workflow Complete ({duration:.0f}ms) ===")

        return {
            "success": True,
            "steps_completed": steps_completed,
            "total_steps": total_steps,
            "data": report,
            "duration": duration,
        }

    except Exception as error:
        print(f"✗ Workflow failed at step {steps_completed + 1}/{total_steps}")
        print(f"Error: {error}")

        duration = (datetime.now() - start_time).total_seconds() * 1000
        return {
            "success": False,
            "steps_completed": steps_completed,
            "total_steps": total_steps,
            "error": str(error),
            "duration": duration,
        }


# Execute workflow
result = await multi_tool_workflow()
print(f"Final result: {json.dumps(result, indent=2)}")
