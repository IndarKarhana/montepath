"""Minimal MCP-compatible stdio boundary for montepath agent tools.

The implementation is dependency-free and intentionally small. It speaks the
JSON-RPC message shape used by MCP clients for initialization, tool discovery,
and tool calls while delegating all domain behavior to the stable agent tool
functions in :mod:`montepath.agent`.
"""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from importlib import metadata
from typing import Any, Callable, Mapping, TextIO

from .agent import (
    agent_benchmark,
    agent_compare,
    agent_compare_methods,
    agent_cost_frontier,
    agent_execute,
    agent_mlmc_calibration,
    agent_plan,
    agent_planner_evidence,
    agent_recommend,
    agent_reproduce,
    agent_tool_manifest,
    agent_validate,
    agent_why_not_faster,
    export_json_schemas,
)

JSONRPC_VERSION = "2.0"
MCP_PROTOCOL_VERSION = "2024-11-05"
SERVER_NAME = "montepath"
MAX_REQUEST_BYTES = 1_000_000
MAX_CONFIG_PATHS = 1_000_000
MAX_BENCHMARK_PROFILE = "full"

ToolHandler = Callable[[Mapping[str, Any]], dict[str, Any]]


@dataclass(frozen=True)
class McpServerLimits:
    max_request_bytes: int = MAX_REQUEST_BYTES
    max_config_paths: int = MAX_CONFIG_PATHS
    max_benchmark_profile: str = MAX_BENCHMARK_PROFILE
    benchmark_execution_requires_opt_in: bool = True

    def as_dict(self) -> dict[str, Any]:
        return {
            "max_request_bytes": self.max_request_bytes,
            "max_config_paths": self.max_config_paths,
            "max_benchmark_profile": self.max_benchmark_profile,
            "benchmark_execution_requires_opt_in": self.benchmark_execution_requires_opt_in,
        }


TOOL_HANDLERS: dict[str, ToolHandler] = {
    "montepath.validate": agent_validate,
    "montepath.recommend": agent_recommend,
    "montepath.plan": agent_plan,
    "montepath.execute": agent_execute,
    "montepath.compare": agent_compare,
    "montepath.benchmark": agent_benchmark,
    "montepath.reproduce": agent_reproduce,
    "montepath.planner_evidence": agent_planner_evidence,
    "montepath.cost_frontier": agent_cost_frontier,
    "montepath.compare_methods": agent_compare_methods,
    "montepath.why_not_faster": agent_why_not_faster,
    "montepath.mlmc_calibration": agent_mlmc_calibration,
}


def server_metadata() -> dict[str, Any]:
    """Return version, capability, and execution-limit metadata."""

    return {
        "schema_version": "mcp-server.v1",
        "name": SERVER_NAME,
        "package": "montepath",
        "version": _package_version(),
        "protocol_version": MCP_PROTOCOL_VERSION,
        "limits": McpServerLimits().as_dict(),
        "failure_policy": {
            "jsonrpc_errors": "protocol and malformed-request failures",
            "tool_failures": "tool calls return ok=false with diagnostics when possible",
            "unsupported_behavior": "unsupported workloads and unavailable capabilities are explicit",
        },
    }


def mcp_tools() -> list[dict[str, Any]]:
    """Return MCP-style tool descriptors with embedded input schemas."""

    schemas = export_json_schemas()
    tools = []
    for tool in agent_tool_manifest()["tools"]:
        tools.append(
            {
                "name": tool["name"],
                "description": tool["description"],
                "inputSchema": schemas.get(tool["input_schema"], {"type": "object"}),
                "annotations": {
                    "determinism": tool["determinism"],
                    "failure_mode": tool["failure_mode"],
                    "output_schema": tool["output_schema"],
                },
            }
        )
    return tools


def handle_jsonrpc(message: Mapping[str, Any]) -> dict[str, Any] | None:
    """Handle one JSON-RPC message.

    Notifications return ``None``. Requests return a JSON-RPC response.
    """

    request_id = message.get("id")
    method = message.get("method")
    if request_id is None:
        if method in {"notifications/initialized", "$/cancelRequest"}:
            return None
        return None
    if message.get("jsonrpc") != JSONRPC_VERSION:
        return _error(request_id, -32600, "Invalid Request", "jsonrpc must be '2.0'")
    if not isinstance(method, str):
        return _error(request_id, -32600, "Invalid Request", "method must be a string")

    try:
        if method == "initialize":
            return _result(
                request_id,
                {
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "serverInfo": {
                        "name": SERVER_NAME,
                        "version": _package_version(),
                    },
                    "capabilities": {"tools": {"listChanged": False}},
                    "metadata": server_metadata(),
                },
            )
        if method == "tools/list":
            return _result(request_id, {"tools": mcp_tools(), "metadata": server_metadata()})
        if method == "tools/call":
            return _result(request_id, _call_tool(message.get("params") or {}))
        if method in {"ping", "health"}:
            return _result(request_id, {"ok": True, "metadata": server_metadata()})
    except Exception as exc:  # pragma: no cover - final guard for process boundary.
        return _error(request_id, -32603, "Internal error", f"{exc.__class__.__name__}: {exc}")

    return _error(request_id, -32601, "Method not found", f"Unsupported method {method!r}")


def serve_stdio(stdin: TextIO = sys.stdin, stdout: TextIO = sys.stdout) -> int:
    """Serve newline-delimited JSON-RPC messages over stdio."""

    for line in stdin:
        if len(line.encode("utf-8")) > MAX_REQUEST_BYTES:
            response = _error(None, -32600, "Invalid Request", "request exceeds size limit")
        else:
            try:
                message = json.loads(line)
            except json.JSONDecodeError as exc:
                response = _error(None, -32700, "Parse error", str(exc))
            else:
                if not isinstance(message, Mapping):
                    response = _error(None, -32600, "Invalid Request", "message must be an object")
                else:
                    response = handle_jsonrpc(message)
        if response is not None:
            stdout.write(json.dumps(response, separators=(",", ":")) + "\n")
            stdout.flush()
    return 0


def main() -> int:
    """Console entry point for the MCP stdio server."""

    return serve_stdio()


def _call_tool(params: Mapping[str, Any]) -> dict[str, Any]:
    if not isinstance(params, Mapping):
        return _tool_failure("MC_MCP_INVALID_PARAMS", "params must be an object")
    name = params.get("name")
    arguments = params.get("arguments") or {}
    if not isinstance(name, str):
        return _tool_failure("MC_MCP_INVALID_TOOL", "tool name must be a string")
    if not isinstance(arguments, Mapping):
        return _tool_failure("MC_MCP_INVALID_ARGUMENTS", "tool arguments must be an object")
    handler = TOOL_HANDLERS.get(name)
    if handler is None:
        return _tool_failure("MC_MCP_UNKNOWN_TOOL", f"unknown tool {name!r}")

    diagnostics = _enforce_limits(name, arguments)
    if diagnostics:
        payload = {"ok": False, "diagnostics": diagnostics, "metadata": server_metadata()}
    else:
        payload = handler(arguments)
    return {
        "content": [
            {
                "type": "text",
                "text": json.dumps(payload, sort_keys=True),
            }
        ],
        "isError": not bool(payload.get("ok", False)),
        "metadata": server_metadata(),
    }


def _enforce_limits(name: str, arguments: Mapping[str, Any]) -> list[dict[str, str]]:
    diagnostics: list[dict[str, str]] = []
    config = arguments.get("config")
    if isinstance(config, Mapping):
        n_paths = config.get("n_paths")
        if isinstance(n_paths, int) and n_paths > MAX_CONFIG_PATHS:
            diagnostics.append(
                {
                    "code": "MC_MCP_LIMIT_PATHS",
                    "message": f"n_paths={n_paths} exceeds MCP limit {MAX_CONFIG_PATHS}",
                    "suggestion": "Run large jobs through benchmark/native APIs outside the MCP tool boundary.",
                }
            )
    if name == "montepath.benchmark" and arguments.get("execute") and arguments.get("profile") == "full":
        diagnostics.append(
            {
                "code": "MC_MCP_LIMIT_BENCHMARK",
                "message": "full benchmark execution is not allowed through this MCP boundary",
                "suggestion": "Use dry-run benchmark metadata or run the benchmark harness directly.",
            }
        )
    return diagnostics


def _tool_failure(code: str, message: str) -> dict[str, Any]:
    return {
        "content": [
            {
                "type": "text",
                "text": json.dumps(
                    {
                        "ok": False,
                        "diagnostics": [
                            {
                                "code": code,
                                "message": message,
                                "suggestion": "Check the MCP tool manifest and request schema.",
                            }
                        ],
                    },
                    sort_keys=True,
                ),
            }
        ],
        "isError": True,
        "metadata": server_metadata(),
    }


def _result(request_id: Any, result: Mapping[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": JSONRPC_VERSION, "id": request_id, "result": dict(result)}


def _error(request_id: Any, code: int, message: str, detail: str) -> dict[str, Any]:
    return {
        "jsonrpc": JSONRPC_VERSION,
        "id": request_id,
        "error": {
            "code": code,
            "message": message,
            "data": {"detail": detail, "metadata": server_metadata()},
        },
    }


def _package_version() -> str:
    try:
        return metadata.version("montepath")
    except metadata.PackageNotFoundError:
        return "0.1.0"


if __name__ == "__main__":
    raise SystemExit(main())
