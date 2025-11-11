/**
 * MCP Client Library for TypeScript/Deno
 *
 * Local implementation of MCP protocol client for calling MCP tools from code.
 * This allows subagents to write code that composes multiple MCP tool calls
 * without loading all tool schemas into the main context.
 *
 * Usage:
 *   import { callMCPTool } from './lib/mcp-client.ts';
 *
 *   const result = await callMCPTool('mcp__filesystem__readFile', {
 *     path: '/tmp/data.json'
 *   });
 */

interface MCPToolCall {
  tool: string;
  parameters: Record<string, unknown>;
}

interface MCPToolResult {
  success: boolean;
  data?: unknown;
  error?: string;
}

interface MCPServerConfig {
  command: string;
  args?: string[];
}

/**
 * Parse MCP tool name into server and tool components
 */
function parseMCPToolName(toolName: string): { server: string; tool: string } {
  const parts = toolName.split("__");

  if (parts.length !== 3 || parts[0] !== "mcp") {
    throw new Error(
      `Invalid MCP tool name format: ${toolName}. Expected: mcp__<server>__<tool>`,
    );
  }

  return {
    server: parts[1],
    tool: parts[2],
  };
}

/**
 * Get MCP server configuration from environment or config file
 */
async function getMCPServerConfig(
  serverName: string,
): Promise<MCPServerConfig> {
  // Try environment variable first
  const configPath = Deno.env.get("MCP_CONFIG_PATH") ||
    Deno.env.get("HOME") + "/.mcp.json";

  try {
    const configContent = await Deno.readTextFile(configPath);
    const config = JSON.parse(configContent);

    if (!config.mcpServers || !config.mcpServers[serverName]) {
      throw new Error(
        `MCP server '${serverName}' not found in config: ${configPath}`,
      );
    }

    return config.mcpServers[serverName];
  } catch (error) {
    const err = error as Error;
    throw new Error(
      `Failed to load MCP config from ${configPath}: ${err.message}`,
    );
  }
}

/**
 * Call an MCP tool via stdio protocol
 */
async function callMCPToolViaStdio(
  serverConfig: MCPServerConfig,
  toolName: string,
  parameters: Record<string, unknown>,
): Promise<unknown> {
  const command = new Deno.Command(serverConfig.command, {
    args: serverConfig.args || [],
    stdin: "piped",
    stdout: "piped",
    stderr: "piped",
  });

  const process = command.spawn();
  const writer = process.stdin.getWriter();

  // Send MCP request
  const request = {
    jsonrpc: "2.0",
    id: crypto.randomUUID(),
    method: "tools/call",
    params: {
      name: toolName,
      arguments: parameters,
    },
  };

  const encoder = new TextEncoder();
  await writer.write(encoder.encode(JSON.stringify(request) + "\n"));
  await writer.close();

  // Read response
  const output = await process.output();
  const decoder = new TextDecoder();
  const stdout = decoder.decode(output.stdout);
  const stderr = decoder.decode(output.stderr);

  if (stderr) {
    console.error("MCP server stderr:", stderr);
  }

  // Parse JSON-RPC response
  const lines = stdout.trim().split("\n");
  for (const line of lines) {
    if (!line.trim()) continue;

    try {
      const response = JSON.parse(line);

      if (response.error) {
        throw new Error(`MCP tool error: ${JSON.stringify(response.error)}`);
      }

      if (response.result) {
        return response.result;
      }
    } catch (_parseError) {
      // Skip non-JSON lines (might be server logs)
      continue;
    }
  }

  throw new Error("No valid MCP response received");
}

/**
 * Main function to call an MCP tool
 *
 * @param toolName - Full MCP tool name (e.g., 'mcp__filesystem__readFile')
 * @param parameters - Tool parameters as object
 * @returns Tool result
 *
 * @example
 * ```typescript
 * const content = await callMCPTool('mcp__filesystem__readFile', {
 *   path: '/tmp/data.json'
 * });
 * ```
 */
export async function callMCPTool(
  toolName: string,
  parameters: Record<string, unknown> = {},
): Promise<unknown> {
  const { server, tool } = parseMCPToolName(toolName);

  // Get server configuration
  const serverConfig = await getMCPServerConfig(server);

  // Call tool via stdio (most common MCP transport)
  // Future: Add support for HTTP transport
  try {
    const result = await callMCPToolViaStdio(serverConfig, tool, parameters);
    return result;
  } catch (error) {
    const err = error as Error;
    throw new Error(
      `Failed to call MCP tool ${toolName}: ${err.message}`,
    );
  }
}

/**
 * Call multiple MCP tools in parallel
 *
 * @param calls - Array of tool calls
 * @returns Array of results in same order
 *
 * @example
 * ```typescript
 * const [file1, file2] = await callMCPToolsParallel([
 *   { tool: 'mcp__filesystem__readFile', parameters: { path: '/tmp/a.json' } },
 *   { tool: 'mcp__filesystem__readFile', parameters: { path: '/tmp/b.json' } }
 * ]);
 * ```
 */
export async function callMCPToolsParallel(
  calls: MCPToolCall[],
): Promise<unknown[]> {
  return await Promise.all(
    calls.map(({ tool, parameters }) => callMCPTool(tool, parameters)),
  );
}

/**
 * Call multiple MCP tools in parallel with graceful error handling
 *
 * @param calls - Array of tool calls
 * @returns Array of results (settled promises)
 *
 * @example
 * ```typescript
 * const results = await callMCPToolsParallelSettled([
 *   { tool: 'mcp__api__fetch', parameters: { url: '...' } },
 *   { tool: 'mcp__api__fetch', parameters: { url: '...' } }
 * ]);
 *
 * results.forEach((result, i) => {
 *   if (result.status === 'fulfilled') {
 *     console.log(`Success: ${result.value}`);
 *   } else {
 *     console.error(`Failed: ${result.reason}`);
 *   }
 * });
 * ```
 */
export async function callMCPToolsParallelSettled(
  calls: MCPToolCall[],
): Promise<PromiseSettledResult<unknown>[]> {
  return await Promise.allSettled(
    calls.map(({ tool, parameters }) => callMCPTool(tool, parameters)),
  );
}
