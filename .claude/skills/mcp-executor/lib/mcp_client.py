"""
MCP Client Library for Python

Local implementation of MCP protocol client for calling MCP tools from code.
This allows subagents to write code that composes multiple MCP tool calls
without loading all tool schemas into the main context.

Usage:
    from lib.mcp_client import call_mcp_tool

    result = await call_mcp_tool('mcp__filesystem__readFile', {
        'path': '/tmp/data.json'
    })
"""

import asyncio
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple
from uuid import uuid4


class MCPError(Exception):
    """MCP tool call error"""

    pass


def parse_mcp_tool_name(tool_name: str) -> Tuple[str, str]:
    """
    Parse MCP tool name into server and tool components.

    Args:
        tool_name: Full MCP tool name (e.g., 'mcp__filesystem__readFile')

    Returns:
        Tuple of (server_name, tool_name)

    Raises:
        ValueError: If tool name format is invalid
    """
    parts = tool_name.split("__")

    if len(parts) != 3 or parts[0] != "mcp":
        raise ValueError(
            f"Invalid MCP tool name format: {tool_name}. "
            f"Expected: mcp__<server>__<tool>"
        )

    return parts[1], parts[2]


async def get_mcp_server_config(server_name: str) -> Dict[str, Any]:
    """
    Get MCP server configuration from environment or config file.

    Args:
        server_name: Name of the MCP server

    Returns:
        Server configuration dict

    Raises:
        FileNotFoundError: If config file not found
        MCPError: If server not configured
    """
    # Try environment variable first
    config_path_str = os.getenv("MCP_CONFIG_PATH")
    if not config_path_str:
        config_path_str = os.path.join(os.path.expanduser("~"), ".mcp.json")

    config_path = Path(config_path_str)

    if not config_path.exists():
        raise FileNotFoundError(f"MCP config file not found: {config_path}")

    try:
        with open(config_path, "r") as f:
            config = json.load(f)
    except json.JSONDecodeError as e:
        raise MCPError(f"Invalid JSON in MCP config: {e}")

    if "mcpServers" not in config:
        raise MCPError(f"No 'mcpServers' key in config: {config_path}")

    if server_name not in config["mcpServers"]:
        raise MCPError(f"MCP server '{server_name}' not found in config: {config_path}")

    return config["mcpServers"][server_name]


async def call_mcp_tool_via_stdio(
    server_config: Dict[str, Any], tool_name: str, parameters: Dict[str, Any]
) -> Any:
    """
    Call an MCP tool via stdio protocol.

    Args:
        server_config: Server configuration dict
        tool_name: Tool name (without mcp__ prefix)
        parameters: Tool parameters

    Returns:
        Tool result

    Raises:
        MCPError: If tool call fails
    """
    command = server_config["command"]
    args = server_config.get("args", [])

    # Create MCP request
    request = {
        "jsonrpc": "2.0",
        "id": str(uuid4()),
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": parameters},
    }

    request_json = json.dumps(request) + "\n"

    # Start MCP server process
    try:
        process = await asyncio.create_subprocess_exec(
            command,
            *args,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
    except FileNotFoundError:
        raise MCPError(f"MCP server command not found: {command}")

    # Send request
    stdout, stderr = await process.communicate(request_json.encode("utf-8"))

    if stderr:
        stderr_text = stderr.decode("utf-8", errors="replace")
        print(f"MCP server stderr: {stderr_text}", file=sys.stderr)

    # Parse response
    stdout_text = stdout.decode("utf-8", errors="replace")
    lines = stdout_text.strip().split("\n")

    for line in lines:
        if not line.strip():
            continue

        try:
            response = json.loads(line)

            if "error" in response:
                raise MCPError(f"MCP tool error: {json.dumps(response['error'])}")

            if "result" in response:
                return response["result"]
        except json.JSONDecodeError:
            # Skip non-JSON lines (might be server logs)
            continue

    raise MCPError("No valid MCP response received")


async def call_mcp_tool(
    tool_name: str, parameters: Optional[Dict[str, Any]] = None
) -> Any:
    """
    Call an MCP tool.

    Args:
        tool_name: Full MCP tool name (e.g., 'mcp__filesystem__readFile')
        parameters: Tool parameters as dict

    Returns:
        Tool result

    Example:
        >>> content = await call_mcp_tool('mcp__filesystem__readFile', {
        ...     'path': '/tmp/data.json'
        ... })
    """
    if parameters is None:
        parameters = {}

    server_name, tool = parse_mcp_tool_name(tool_name)

    # Get server configuration
    server_config = await get_mcp_server_config(server_name)

    # Call tool via stdio (most common MCP transport)
    # Future: Add support for HTTP transport
    try:
        result = await call_mcp_tool_via_stdio(server_config, tool, parameters)
        return result
    except Exception as e:
        raise MCPError(f"Failed to call MCP tool {tool_name}: {e}") from e


async def call_mcp_tools_parallel(calls: List[Dict[str, Any]]) -> List[Any]:
    """
    Call multiple MCP tools in parallel.

    Args:
        calls: List of dicts with 'tool' and 'parameters' keys

    Returns:
        List of results in same order

    Example:
        >>> results = await call_mcp_tools_parallel([
        ...     {'tool': 'mcp__filesystem__readFile', 'parameters': {'path': '/tmp/a.json'}},
        ...     {'tool': 'mcp__filesystem__readFile', 'parameters': {'path': '/tmp/b.json'}}
        ... ])
    """
    tasks = [call_mcp_tool(call["tool"], call.get("parameters", {})) for call in calls]
    return await asyncio.gather(*tasks)


async def call_mcp_tools_parallel_safe(
    calls: List[Dict[str, Any]],
) -> List[Dict[str, Any]]:
    """
    Call multiple MCP tools in parallel with graceful error handling.

    Args:
        calls: List of dicts with 'tool' and 'parameters' keys

    Returns:
        List of result dicts with 'success', 'data'/'error' keys

    Example:
        >>> results = await call_mcp_tools_parallel_safe([
        ...     {'tool': 'mcp__api__fetch', 'parameters': {'url': '...'}},
        ...     {'tool': 'mcp__api__fetch', 'parameters': {'url': '...'}}
        ... ])
        >>> for i, result in enumerate(results):
        ...     if result['success']:
        ...         print(f"Success: {result['data']}")
        ...     else:
        ...         print(f"Failed: {result['error']}")
    """
    tasks = [call_mcp_tool(call["tool"], call.get("parameters", {})) for call in calls]

    results = await asyncio.gather(*tasks, return_exceptions=True)

    return [
        (
            {"success": True, "data": result}
            if not isinstance(result, Exception)
            else {"success": False, "error": str(result)}
        )
        for result in results
    ]
