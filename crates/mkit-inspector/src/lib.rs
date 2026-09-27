//! MCP inspector for driving and inspecting GPUI examples.
//!
//! The stdio transport speaks newline-delimited JSON-RPC as specified by MCP.
//! It writes protocol messages only to stdout; diagnostics belong on stderr.

use serde_json::{Value, json};
use std::io::{BufRead, Write};

mod fixture;
pub use fixture::GpuiInspector;

pub const MCP_VERSION: &str = "2026-07-28";
pub const SERVER_NAME: &str = "mkit-inspector";

/// A backend that performs tool operations against the running GPUI fixture.
pub trait InspectorBackend {
    fn call(&mut self, tool: &str, arguments: &Value) -> Result<Value, String>;
}

/// A JSON-RPC MCP server over any pair of line-oriented streams.
pub struct InspectorServer<B> {
    backend: B,
}

impl<B: InspectorBackend> InspectorServer<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Process one complete MCP/JSON-RPC message. Notifications return `None`.
    pub fn handle(&mut self, request: &Value) -> Option<Value> {
        let id = request.get("id")?.clone();
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let params = request.get("params").unwrap_or(&Value::Null);
        let result = match method {
            "server/discover" => Some(discover_result()),
            // The legacy initialization exchange remains supported for older clients.
            "initialize" => Some(json!({
                "protocolVersion": params.get("protocolVersion").and_then(Value::as_str)
                    .filter(|version| *version == MCP_VERSION || *version == "2025-11-25")
                    .unwrap_or(MCP_VERSION),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION")},
                "instructions": "Launch hello, gallery, counter, state_entities, or reactivity_views before using input, screenshot, or inspection tools. The contexts example is nonvisual and is not launchable."
            })),
            "notifications/initialized" | "notifications/cancelled" => return None,
            "tools/list" => Some(json!({
                "resultType": "complete",
                "tools": tool_definitions(),
                "ttlMs": 0,
                "cacheScope": "session"
            })),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params.get("arguments").unwrap_or(&Value::Null);
                match self.backend.call(name, args) {
                    Ok(value) if value.get("_mcpImage").is_some() => {
                        let data = value["_mcpImage"].as_str().unwrap_or("");
                        Some(
                            json!({"resultType":"complete","content":[{"type":"image","data":data,"mimeType":"image/png"}],"isError":false}),
                        )
                    }
                    Ok(value) => Some(
                        json!({"resultType":"complete","content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false}),
                    ),
                    Err(error) => Some(
                        json!({"resultType":"complete","content":[{"type":"text","text":error}],"isError":true}),
                    ),
                }
            }
            _ => {
                return Some(json!({
                    "jsonrpc": "2.0", "id": id,
                    "error": {"code": -32601, "message": format!("method not found: {method}")}
                }));
            }
        }?;
        let mut result = result;
        if method != "initialize"
            && let Some(object) = result.as_object_mut()
        {
            object.insert("_meta".into(), json!({"io.modelcontextprotocol/serverInfo":{"name":SERVER_NAME,"version":env!("CARGO_PKG_VERSION")}}));
        }
        Some(json!({"jsonrpc":"2.0", "id":id, "result":result}))
    }

    /// Serve until stdin closes. Every request and response occupies one line.
    pub fn serve<R: BufRead, W: Write>(
        &mut self,
        mut input: R,
        mut output: W,
    ) -> std::io::Result<()> {
        let mut line = String::new();
        loop {
            line.clear();
            if input.read_line(&mut line)? == 0 {
                break;
            }
            let parsed: Result<Value, _> = serde_json::from_str(&line);
            let response = match parsed {
                Ok(request) => self.handle(&request),
                Err(error) => Some(json!({
                    "jsonrpc":"2.0", "id":null,
                    "error":{"code":-32700,"message":format!("parse error: {error}")}
                })),
            };
            if let Some(response) = response {
                serde_json::to_writer(&mut output, &response)?;
                output.write_all(b"\n")?;
                output.flush()?;
            }
        }
        Ok(())
    }
}

fn discover_result() -> Value {
    json!({
        "resultType":"complete",
        "supportedVersions":[MCP_VERSION],
        "capabilities":{"tools":{}},
        "_meta":{"io.modelcontextprotocol/serverInfo":{"name":SERVER_NAME,"version":env!("CARGO_PKG_VERSION")}},
        "ttlMs":3600000,
        "cacheScope":"public"
    })
}

fn tool_definitions() -> Value {
    json!([
        {"name":"launch","description":"Launch a supported visual book example in the inspector session. The nonvisual contexts example returns a clear error.","inputSchema":{"type":"object","properties":{"example":{"type":"string","enum":["hello","gallery","counter","state_entities","reactivity_views","contexts"],"description":"Visual book example or gallery view to launch; contexts is nonvisual."}},"required":["example"],"additionalProperties":false}},
        {"name":"screenshot","description":"Capture the current GPUI window as PNG image data when the host renderer supports it.","inputSchema":{"type":"object","properties":{},"additionalProperties":false}},
        {"name":"press","description":"Dispatch GPUI keystrokes using space-separated GPUI syntax.","inputSchema":{"type":"object","properties":{"keys":{"type":"string"}},"required":["keys"],"additionalProperties":false}},
        {"name":"type","description":"Type text through GPUI's simulated keyboard input.","inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"],"additionalProperties":false}},
        {"name":"click","description":"Click a named target supported by the running example or a point in the current GPUI window.","inputSchema":{"type":"object","properties":{"target":{"type":"string","description":"Targets include @increment, @strong-increment, and @weak-increment depending on the launched example."},"x":{"type":"number"},"y":{"type":"number"}},"additionalProperties":false}},
        {"name":"a11y_tree","description":"Read GPUI's actual accessibility tree. Returns an error if the platform has not activated accessibility.","inputSchema":{"type":"object","properties":{},"additionalProperties":false}},
        {"name":"entity_state","description":"Read the running fixture's state.","inputSchema":{"type":"object","properties":{},"additionalProperties":false}}
    ])
}

/// Run a server over process stdin/stdout.
pub fn serve_stdio<B: InspectorBackend>(backend: B) -> std::io::Result<()> {
    InspectorServer::new(backend).serve(std::io::stdin().lock(), std::io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingBackend(Vec<String>);
    impl InspectorBackend for RecordingBackend {
        fn call(&mut self, tool: &str, arguments: &Value) -> Result<Value, String> {
            if tool == "unsupported" {
                return Err("unknown tool".into());
            }
            self.0.push(tool.into());
            Ok(json!({"tool":tool,"arguments":arguments}))
        }
    }

    #[test]
    fn initialize_lists_tools_and_calls_backend() {
        let mut server = InspectorServer::new(RecordingBackend::default());
        let initialized = server.handle(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":MCP_VERSION}})).unwrap();
        assert_eq!(initialized["result"]["protocolVersion"], MCP_VERSION);
        let meta = json!({"io.modelcontextprotocol/protocolVersion":MCP_VERSION,"io.modelcontextprotocol/clientCapabilities":{}});
        let listed = server
            .handle(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{"_meta":meta}}))
            .unwrap();
        let names = listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            ["launch", "screenshot", "press", "type", "click", "a11y_tree", "entity_state"]
        );
        let called = server.handle(&json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"launch","arguments":{"example":"hello"},"_meta":{"io.modelcontextprotocol/protocolVersion":MCP_VERSION,"io.modelcontextprotocol/clientCapabilities":{}}}})).unwrap();
        assert_eq!(called["result"]["structuredContent"]["tool"], "launch");
    }

    #[test]
    fn supports_modern_discovery_and_returns_tool_errors_as_is_error() {
        let mut server = InspectorServer::new(RecordingBackend::default());
        let discovered =
            server.handle(&json!({"jsonrpc":"2.0","id":1,"method":"server/discover"})).unwrap();
        assert_eq!(discovered["result"]["supportedVersions"][0], MCP_VERSION);
        let failed = server.handle(&json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"unsupported"}})).unwrap();
        assert_eq!(failed["result"]["isError"], true);
    }

    #[test]
    fn stdio_is_newline_delimited_and_suppresses_notifications() {
        let input = b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n";
        let mut output = Vec::new();
        InspectorServer::new(RecordingBackend::default()).serve(&input[..], &mut output).unwrap();
        assert_eq!(output.iter().filter(|byte| **byte == b'\n').count(), 1);
        let response: Value = serde_json::from_slice(&output[..output.len() - 1]).unwrap();
        assert_eq!(response["id"], 1);
    }
}
