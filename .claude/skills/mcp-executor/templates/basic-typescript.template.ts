/**
 * Template: Basic TypeScript MCP Tool Call
 * Purpose: Minimal starting point for single tool call
 * Use Case: When you need to call one MCP tool and process its result
 *
 * How to Adapt:
 * 1. Replace 'mcp__server__tool' with your actual tool name
 * 2. Update parameters object with required parameters for your tool
 * 3. Add your custom processing logic in the process section
 * 4. Customize error handling for your specific needs
 * 5. Update return value structure as needed
 *
 * Example Usage:
 * Copy this template, rename it, and modify for your specific use case
 *
 * Execution:
 * deno run --allow-read --allow-run --allow-env your-script.ts
 */

import { callMCPTool } from "../lib/mcp-client.ts";

// TODO: Replace with your tool name (format: mcp__server__tool)
const TOOL_NAME = "mcp__server__tool";

// TODO: Define parameters for your tool call
const parameters = {
  // Add your parameters here
  // Example:
  // path: '/tmp/data.json',
  // encoding: 'utf-8'
};

try {
  console.log(`Calling ${TOOL_NAME}...`);

  // Step 1: Call MCP tool
  const result = await callMCPTool(TOOL_NAME, parameters);

  console.log("Tool call successful");

  // Step 2: TODO: Process the result
  // Add your custom logic here
  const processed = result; // Replace with actual processing

  // Step 3: TODO: Return your result
  return {
    success: true,
    data: processed,
    // Add any additional fields you need
  };
} catch (error) {
  // Step 4: Handle errors
  console.error(`Error calling ${TOOL_NAME}:`, error.message);

  return {
    success: false,
    error: error.message,
  };
}
