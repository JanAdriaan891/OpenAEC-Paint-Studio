//! Newline-delimited MCP stdio transport. stdout contains JSON-RPC only.
use anyhow::{ensure, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use paint_core::{Document, MAX_BYTES, VERSION};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
pub fn tools() -> Vec<Value> {
    let id = json!({"type":"string","format":"uuid"});
    let rev = json!({"type":"integer","minimum":1});
    let schema = |properties: Value, required: Vec<&str>| json!({"type":"object","properties":properties,"required":required,"additionalProperties":false});
    vec![
        json!({"name":"paint_list_documents","description":"List documents stored in the local Paint API workspace.","inputSchema":schema(json!({}),vec![])}),
        json!({"name":"paint_create_document","description":"Create a blank Paint document and return UUID and revision.","inputSchema":schema(json!({"name":{"type":"string","minLength":1,"maxLength":120},"width":{"type":"integer","minimum":1,"maximum":4096,"default":1200},"height":{"type":"integer","minimum":1,"maximum":4096,"default":800}}),vec!["name"])}),
        json!({"name":"paint_get_document","description":"Read a document and its revision before editing.","inputSchema":schema(json!({"id":id}),vec!["id"])}),
        json!({"name":"paint_draw","description":"Append brush, eraser, fill, line, rectangle, ellipse, text or image operations. Coordinates in canvas pixels; colors #RRGGBB; size 1–200; opacity 0–1. Requires the revision just read.","inputSchema":schema(json!({"id":id,"expectedRevision":rev,"operations":{"type":"array","minItems":1,"maxItems":200,"items":{"type":"object","properties":{"type":{"enum":["brush","eraser","fill","line","rectangle","ellipse","text","image"]},"color":{"type":"string","pattern":"^#[0-9a-fA-F]{6}$"},"size":{"type":"number","minimum":1,"maximum":200},"opacity":{"type":"number","minimum":0,"maximum":1},"points":{"type":"array","items":{"type":"object","properties":{"x":{"type":"number"},"y":{"type":"number"}},"required":["x","y"]}},"from":{"type":"object"},"to":{"type":"object"},"at":{"type":"object"},"filled":{"type":"boolean"},"text":{"type":"string"},"fontFamily":{"enum":["sans-serif","serif","monospace"]},"bold":{"type":"boolean"},"transform":{"type":"array","minItems":6,"maxItems":6,"items":{"type":"number"}},"data":{"type":"string"},"sourceSize":{"type":"object"}},"required":["type"]}}}),vec!["id","expectedRevision","operations"])}),
        json!({"name":"paint_undo","description":"Remove the last operation using the current revision.","inputSchema":schema(json!({"id":id,"expectedRevision":rev}),vec!["id","expectedRevision"])}),
        json!({"name":"paint_export_png","description":"Render a document to an inline PNG image.","inputSchema":schema(json!({"id":id}),vec!["id"])}),
    ]
}
pub struct Client {
    base: String,
    token: String,
    http: reqwest::Client,
}
impl Client {
    pub fn new(base: String, token: String) -> Result<Self> {
        let url = reqwest::Url::parse(&base)?;
        ensure!(
            url.scheme() == "http"
                && url.username().is_empty()
                && url.password().is_none()
                && matches!(
                    url.host_str(),
                    Some("127.0.0.1" | "localhost" | "[::1]" | "::1")
                ),
            "PAINT_API_URL must be local HTTP"
        );
        Ok(Self {
            base: base.trim_end_matches('/').into(),
            token,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }
    async fn call(&self, name: &str, args: &Value) -> Result<Value> {
        let id = if matches!(
            name,
            "paint_get_document" | "paint_draw" | "paint_undo" | "paint_export_png"
        ) {
            uuid::Uuid::parse_str(args["id"].as_str().unwrap_or(""))?.to_string()
        } else {
            String::new()
        };
        let (method, path, body) = match name {
            "paint_list_documents" => (reqwest::Method::GET, "/v1/documents".into(), None),
            "paint_create_document" => {
                let w = args["width"].as_u64().unwrap_or(1200);
                let h = args["height"].as_u64().unwrap_or(800);
                ensure!(w <= 4096 && h <= 4096, "Invalid dimensions");
                let d = Document::new(w as u32, h as u32, args["name"].as_str().unwrap_or(""));
                d.validate()?;
                (
                    reqwest::Method::POST,
                    "/v1/documents".into(),
                    Some(serde_json::to_value(d)?),
                )
            }
            "paint_get_document" => (reqwest::Method::GET, format!("/v1/documents/{id}"), None),
            "paint_draw" => (
                reqwest::Method::POST,
                format!("/v1/documents/{id}/operations"),
                Some(
                    json!({"operations":args["operations"],"expectedRevision":args["expectedRevision"]}),
                ),
            ),
            "paint_undo" => (
                reqwest::Method::POST,
                format!("/v1/documents/{id}/undo"),
                Some(json!({"expectedRevision":args["expectedRevision"]})),
            ),
            "paint_export_png" => (
                reqwest::Method::GET,
                format!("/v1/documents/{id}/export.png"),
                None,
            ),
            _ => anyhow::bail!("Unknown tool"),
        };
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.base))
            .bearer_auth(&self.token);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        ensure!(bytes.len() <= MAX_BYTES, "Response exceeds 64 MB");
        if !status.is_success() {
            anyhow::bail!(
                "{}",
                serde_json::from_slice::<Value>(&bytes)
                    .ok()
                    .and_then(|v| v["error"].as_str().map(str::to_owned))
                    .unwrap_or_else(|| format!("API {status}"))
            );
        }
        if name == "paint_export_png" {
            Ok(
                json!({"content":[{"type":"image","mimeType":"image/png","data":STANDARD.encode(bytes)}]}),
            )
        } else {
            let value: Value = serde_json::from_slice(&bytes)?;
            Ok(json!({"content":[{"type":"text","text":serde_json::to_string(&value)?}]}))
        }
    }
    pub async fn handle(&self, request: &Value, initialized: &mut bool) -> Option<Value> {
        let method = request["method"].as_str().unwrap_or("");
        if request.get("id").is_none() {
            return None;
        }
        let id = &request["id"];
        let result = match method {
            "initialize" => {
                *initialized = true;
                let requested = request["params"]["protocolVersion"].as_str().unwrap_or("");
                let version = if ["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"]
                    .contains(&requested)
                {
                    requested
                } else {
                    "2025-11-25"
                };
                json!({"protocolVersion":version,"capabilities":{"tools":{}},"serverInfo":{"name":"openaec-paint-studio","version":VERSION}})
            }
            "ping" => json!({}),
            _ if !*initialized => {
                return Some(
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":-32002,"message":"Initialize first"}}),
                )
            }
            "tools/list" => json!({"tools":tools()}),
            "tools/call" => match self
                .call(
                    request["params"]["name"].as_str().unwrap_or(""),
                    &request["params"]["arguments"],
                )
                .await
            {
                Ok(v) => v,
                Err(e) => json!({"content":[{"type":"text","text":e.to_string()}],"isError":true}),
            },
            _ => {
                return Some(
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not found"}}),
                )
            }
        };
        Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
    }
}
pub async fn serve(client: Client) -> Result<()> {
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut out = tokio::io::stdout();
    let mut initialized = false;
    loop {
        let mut line = Vec::new();
        let read = (&mut reader)
            .take((MAX_BYTES + 1) as u64)
            .read_until(b'\n', &mut line)
            .await?;
        if read == 0 {
            break;
        }
        ensure!(line.len() <= MAX_BYTES, "MCP message exceeds 64 MB");
        let response = match serde_json::from_slice::<Value>(&line) {
            Ok(request) => client.handle(&request, &mut initialized).await,
            Err(_) => Some(
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}),
            ),
        };
        if let Some(response) = response {
            out.write_all(serde_json::to_string(&response)?.as_bytes())
                .await?;
            out.write_all(b"\n").await?;
            out.flush().await?;
        }
    }
    Ok(())
}
