#[tokio::main]
async fn main()->anyhow::Result<()>{let dir=paint_services::store::data_directory();let token=paint_services::store::api_token(&dir)?;let base=std::env::var("PAINT_API_URL").unwrap_or_else(|_|"http://127.0.0.1:47831".into());paint_services::mcp::serve(paint_services::mcp::Client::new(base,token)?).await}
