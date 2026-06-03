import json
import unittest

from montepath.mcp_server import handle_jsonrpc, mcp_tools, server_metadata


class McpServerTests(unittest.TestCase):
    def test_server_metadata_declares_limits_and_version(self) -> None:
        metadata = server_metadata()

        self.assertEqual(metadata["schema_version"], "mcp-server.v1")
        self.assertEqual(metadata["name"], "montepath")
        self.assertIn("limits", metadata)
        self.assertTrue(metadata["limits"]["benchmark_execution_requires_opt_in"])

    def test_tools_list_exposes_agent_tools_with_schemas(self) -> None:
        tools = mcp_tools()
        names = {tool["name"] for tool in tools}

        self.assertIn("montepath.validate", names)
        self.assertIn("montepath.execute", names)
        self.assertIn("montepath.mlmc_calibration", names)
        self.assertIn("montepath.capabilities", names)
        self.assertIn("montepath.production_check", names)
        self.assertIn("montepath.validation_report", names)
        validate = next(tool for tool in tools if tool["name"] == "montepath.validate")
        self.assertEqual(validate["inputSchema"]["type"], "object")
        self.assertIn("annotations", validate)

    def test_initialize_and_tools_list_jsonrpc(self) -> None:
        initialize = handle_jsonrpc(
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}
        )
        listed = handle_jsonrpc({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})

        self.assertEqual(initialize["result"]["serverInfo"]["name"], "montepath")
        self.assertEqual(listed["result"]["tools"][0]["name"], "montepath.validate")

    def test_tools_call_returns_mcp_content(self) -> None:
        response = handle_jsonrpc(
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "montepath.validate",
                    "arguments": {
                        "workload": "european_call",
                        "config": {"n_paths": 128, "n_steps": 4, "seed": 9},
                    },
                },
            }
        )

        payload = json.loads(response["result"]["content"][0]["text"])
        self.assertTrue(payload["ok"])
        self.assertFalse(response["result"]["isError"])

    def test_unknown_tool_and_limit_failures_are_structured(self) -> None:
        unknown = handle_jsonrpc(
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {"name": "montepath.nope", "arguments": {}},
            }
        )
        limited = handle_jsonrpc(
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "montepath.execute",
                    "arguments": {
                        "workload": "european_call",
                        "config": {"n_paths": 1_000_001},
                    },
                },
            }
        )

        unknown_payload = json.loads(unknown["result"]["content"][0]["text"])
        limited_payload = json.loads(limited["result"]["content"][0]["text"])
        self.assertEqual(unknown_payload["diagnostics"][0]["code"], "MC_MCP_UNKNOWN_TOOL")
        self.assertEqual(limited_payload["diagnostics"][0]["code"], "MC_MCP_LIMIT_PATHS")
        self.assertTrue(unknown["result"]["isError"])
        self.assertTrue(limited["result"]["isError"])


if __name__ == "__main__":
    unittest.main()
