use paint_core::{Document, VERSION};
use paint_services::{api, mcp::Client, store::Store};
use serde_json::{json, Value};
#[tokio::test(flavor = "multi_thread")]
async fn api_and_mcp_share_the_real_renderer_and_revision_control() {
    let dir = tempfile::tempdir().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            api::router(
                Store::new(dir.path().join("documents")).unwrap(),
                "test-token".into(),
            ),
        )
        .await
        .unwrap();
    });
    let base = format!("http://{address}");
    let http = reqwest::Client::new();
    assert_eq!(
        http.get(format!("{base}/health"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let authorized = |path: &str| http.get(format!("{base}{path}")).bearer_auth("test-token");
    let health: Value = authorized("/health")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(health["version"], VERSION);
    assert_eq!(
        authorized("/health")
            .header("Origin", "https://hostile.example")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        authorized("/health")
            .header("Host", "hostile.example")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let client = Client::new(base.clone(), "test-token".into()).unwrap();
    let mut initialized = false;
    let init=client.handle(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),&mut initialized).await.unwrap();
    assert_eq!(init["result"]["serverInfo"]["version"], VERSION);
    let tools = client
        .handle(
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            &mut initialized,
        )
        .await
        .unwrap();
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 6);
    let create=client.handle(&json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"paint_create_document","arguments":{"name":"MCP Raster","width":64,"height":64}}}),&mut initialized).await.unwrap();
    let created: Value =
        serde_json::from_str(create["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    let id = created["id"].as_str().unwrap();
    let draw=client.handle(&json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"paint_draw","arguments":{"id":id,"expectedRevision":1,"operations":[{"type":"rectangle","from":{"x":5,"y":5},"to":{"x":50,"y":50},"size":2,"opacity":1,"color":"#d97706","filled":true}]}}}),&mut initialized).await.unwrap();
    assert_ne!(draw["result"]["isError"], true);
    let conflict = http
        .put(format!("{base}/v1/documents/{id}"))
        .bearer_auth("test-token")
        .json(&json!({"document":Document::new(64,64,"Stale"),"expectedRevision":1}))
        .send()
        .await
        .unwrap();
    assert_eq!(conflict.status(), 409);
    let png=client.handle(&json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"paint_export_png","arguments":{"id":id}}}),&mut initialized).await.unwrap();
    assert_eq!(png["result"]["content"][0]["type"], "image");
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(png["result"]["content"][0]["data"].as_str().unwrap())
        .unwrap();
    let p = paint_core::codec::decode_png(&bytes).unwrap();
    assert_eq!(p.pixel(20, 20).unwrap().demultiply().red(), 217);
    let undo=client.handle(&json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"paint_undo","arguments":{"id":id,"expectedRevision":2}}}),&mut initialized).await.unwrap();
    assert_ne!(undo["result"]["isError"], true);
    let stored: Value = authorized(&format!("/v1/documents/{id}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stored["revision"], 3);
    assert_eq!(
        stored["document"]["operations"].as_array().unwrap().len(),
        0
    );
    let invalid = http
        .post(format!("{base}/v1/documents"))
        .bearer_auth("test-token")
        .json(&Document::new(4096, 4096, "Too big"))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), 400);
    server.abort();
}
#[test]
fn stored_documents_remain_readable_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path()).unwrap();
    let doc = store.create(Document::new(16, 16, "Persisted")).unwrap();
    drop(store);
    let store = Store::new(dir.path()).unwrap();
    assert_eq!(store.get(doc.id).unwrap().document.name, "Persisted");
}
#[test]
fn mcp_refuses_remote_api_urls() {
    assert!(Client::new("https://127.0.0.1:47831".into(), "token".into()).is_err());
    assert!(Client::new("http://example.com".into(), "token".into()).is_err());
    assert!(Client::new("http://user:secret@localhost:47831".into(), "token".into()).is_err());
}

#[tokio::test]
async fn standalone_mcp_uses_only_newline_json_on_stdout() {
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;
    let directory = tempfile::tempdir().unwrap();
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_paint-mcp"))
        .env("PAINT_DATA_DIR", directory.path())
        .env("PAINT_API_TOKEN", "stdio-test-token")
        .env("PAINT_API_URL", "http://127.0.0.1:47831")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for message in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"ping"}),
    ] {
        stdin
            .write_all(format!("{message}\n").as_bytes())
            .await
            .unwrap();
    }
    stdin.write_all(b"invalid json\n").await.unwrap();
    drop(stdin);
    let output = tokio::time::timeout(std::time::Duration::from_secs(10), child.wait_with_output())
        .await
        .unwrap()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let messages: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0]["result"]["serverInfo"]["version"], VERSION);
    assert_eq!(messages[1]["result"]["tools"].as_array().unwrap().len(), 6);
    assert_eq!(messages[2]["result"], json!({}));
    assert_eq!(messages[3]["error"]["code"], -32700);
}
