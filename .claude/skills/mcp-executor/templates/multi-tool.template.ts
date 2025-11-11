/**
 * Template: Multi-Tool Composition
 * Purpose: Template for calling multiple MCP tools in sequence or parallel
 * Use Case: When you need to compose multiple tool operations together
 *
 * How to Adapt:
 * 1. Define your tool calls in the tools array
 * 2. Choose sequential or parallel execution pattern
 * 3. Add logic to process and combine results
 * 4. Customize error handling strategy (fail-fast vs. partial success)
 * 5. Update return value with your aggregated data
 *
 * Example Usage:
 * Use for workflows like: fetch data → transform → store results
 *
 * Execution:
 * deno run --allow-read --allow-run --allow-env your-script.ts
 */

// deno-lint-ignore no-unused-vars
import { callMCPTool, callMCPToolsParallel } from "../lib/mcp-client.ts";

// TODO: Define your tool calls
const tools = [
  {
    name: "mcp__server1__tool1",
    params: {/* tool 1 parameters */},
  },
  {
    name: "mcp__server2__tool2",
    params: {/* tool 2 parameters */},
  },
  {
    name: "mcp__server3__tool3",
    params: {/* tool 3 parameters */},
  },
];

console.log(`Executing ${tools.length} tool operations...`);

try {
  // OPTION A: Sequential Execution (one after another)
  // Use when results depend on each other
  /*
  const results = [];
  for (const tool of tools) {
    console.log(`Calling ${tool.name}...`);
    const result = await callMCPTool(tool.name, tool.params);
    results.push(result);

    // TODO: Add logic between calls if needed
    // Example: use result from previous call in next call
  }
  */

  // OPTION B: Parallel Execution (all at once)
  // Use when operations are independent

  // Using helper function for parallel execution
  const results = await callMCPToolsParallel(
    tools.map((tool) => ({
      tool: tool.name,
      parameters: tool.params,
    })),
  );

  console.log("All tool calls completed");

  // TODO: Process and combine results
  const combined = {
    // Example: Combine all results
    results: results,
    // Add your custom aggregation logic
  };

  // TODO: Return aggregated data
  return {
    success: true,
    toolsCalled: tools.length,
    data: combined,
  };
} catch (error) {
  console.error("Error in multi-tool execution:", error.message);

  // TODO: Decide error handling strategy
  // - Fail fast: throw error to stop everything
  // - Partial success: return what succeeded so far

  return {
    success: false,
    error: error.message,
  };
}

// OPTION C: Parallel with Partial Failure Handling
// Use when you want to continue even if some calls fail
/*
const results = await Promise.allSettled(
  tools.map(tool => callMCPTool(tool.name, tool.params))
);

const successes = results.filter(r => r.status === 'fulfilled');
const failures = results.filter(r => r.status === 'rejected');

console.log(`Success: ${successes.length}, Failed: ${failures.length}`);

return {
  success: failures.length === 0,
  successCount: successes.length,
  failureCount: failures.length,
  data: successes.map(r => r.value),
  errors: failures.map(r => r.reason.message)
};
*/
