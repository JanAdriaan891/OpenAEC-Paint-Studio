use paint_core::{Document,VERSION};
use serde_json::{json,Value};
use paint_services::{api,store::Store,mcp::Client};
#[tokio::test(flavor="multi_thread")]
async fn api_and_mcp_share_the_real_renderer_and_revision_control(){
 let dir=tempfile::tempdir().unwrap();let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();let server=tokio::spawn(async move{axum::serve(listener,api::router(Store::new(dir.path().join("documents")).unwrap(),"test-token".into())).await.unwrap();});
 let base=format!("http://{address}");let http=reqwest::Client::new();assert_eq!(http.get(format!("{base}/health")).send().await.unwrap().status(),401);
 let authorized=|path:&str|http.get(format!("{base}{path}")).bearer_auth("test-token");
 let health:Value=authorized("/health").send().await.unwrap().json().await.unwrap();assert_eq!(health["version"],VERSION);
 assert_eq!(authorized("/health").header("Origin","https://hostile.example").send().await.unwrap().status(),403);
 assert_eq!(authorized("/health").header("Host","hostile.example").send().await.unwrap().status(),403);
 let client=Client::new(base.clone(),"test-token".into()).unwrap();let mut initialized=false;
 let init=client.handle(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),&mut initialized).await.unwrap();assert_eq!(init["result"]["serverInfo"]["version"],VERSION);
 let tools=client.handle(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),&mut initialized).await.unwrap();assert_eq!(tools["result"]["tools"].as_array().unwrap().len(),6);
 let create=client.handle(&json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"paint_create_document","arguments":{"name":"MCP Raster","width":64,"height":64}}}),&mut initialized).await.unwrap();
 let created:Value=serde_json::from_str(create["result"]["content"][0]["text"].as_str().unwrap()).unwrap();let id=created["id"].as_str().unwrap();
 let draw=client.handle(&json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"paint_draw","arguments":{"id":id,"expectedRevision":1,"operations":[{"type":"rectangle","from":{"x":5,"y":5},"to":{"x":50,"y":50},"size":2,"opacity":1,"color":"#d97706","filled":true}]}}}),&mut initialized).await.unwrap();assert_ne!(draw["result"]["isError"],true);
 let conflict=http.put(format!("{base}/v1/documents/{id}")).bearer_auth("test-token").json(&json!({"document":Document::new(64,64,"Stale"),"expectedRevision":1})).send().await.unwrap();assert_eq!(conflict.status(),409);
 let png=client.handle(&json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"paint_export_png","arguments":{"id":id}}}),&mut initialized).await.unwrap();assert_eq!(png["result"]["content"][0]["type"],"image");use base64::Engine;let bytes=base64::engine::general_purpose::STANDARD.decode(png["result"]["content"][0]["data"].as_str().unwrap()).unwrap();let p=paint_core::codec::decode_png(&bytes).unwrap();assert_eq!(p.pixel(20,20).unwrap().demultiply().red(),217);
 let undo=client.handle(&json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"paint_undo","arguments":{"id":id,"expectedRevision":2}}}),&mut initialized).await.unwrap();assert_ne!(undo["result"]["isError"],true);
 let stored:Value=authorized(&format!("/v1/documents/{id}")).send().await.unwrap().json().await.unwrap();assert_eq!(stored["revision"],3);assert_eq!(stored["document"]["operations"].as_array().unwrap().len(),0);
 let invalid=http.post(format!("{base}/v1/documents")).bearer_auth("test-token").json(&Document::new(4096,4096,"Too big")).send().await.unwrap();assert_eq!(invalid.status(),400);
 server.abort();
}
#[test]fn stored_documents_remain_readable_across_restarts(){let dir=tempfile::tempdir().unwrap();let store=Store::new(dir.path()).unwrap();let doc=store.create(Document::new(16,16,"Persisted")).unwrap();drop(store);let store=Store::new(dir.path()).unwrap();assert_eq!(store.get(doc.id).unwrap().document.name,"Persisted");}
#[test]fn mcp_refuses_remote_api_urls(){assert!(Client::new("https://127.0.0.1:47831".into(),"token".into()).is_err());assert!(Client::new("http://example.com".into(),"token".into()).is_err());assert!(Client::new("http://user:secret@localhost:47831".into(),"token".into()).is_err());}
