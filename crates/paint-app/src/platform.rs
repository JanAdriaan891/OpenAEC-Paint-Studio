use paint_core::{Document,MAX_BYTES};
use serde::{Serialize,Deserialize};
use openaec_ui::design::ThemeId;
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct Settings{pub theme:ThemeId,#[serde(default)]pub english:bool}
#[cfg(not(target_arch="wasm32"))]
fn directory()->std::path::PathBuf{directories::ProjectDirs::from("org","OpenAEC","PaintStudio").map(|p|p.data_local_dir().to_path_buf()).unwrap_or_else(||std::path::PathBuf::from(".paint-studio"))}
pub fn settings()->Settings{#[cfg(not(target_arch="wasm32"))]{std::fs::read(directory().join("settings.json")).ok().and_then(|b|serde_json::from_slice(&b).ok()).unwrap_or_default()}#[cfg(target_arch="wasm32")]{web_sys::window().and_then(|w|w.local_storage().ok().flatten()).and_then(|s|s.get_item("paint-rust-settings").ok().flatten()).and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default()}}
pub fn store_settings(value:&Settings){if cfg!(test){return;}if let Ok(data)=serde_json::to_string(value){#[cfg(not(target_arch="wasm32"))]{let dir=directory();let _=std::fs::create_dir_all(&dir);let _=std::fs::write(dir.join("settings.json"),data);}#[cfg(target_arch="wasm32")]{if let Some(s)=web_sys::window().and_then(|w|w.local_storage().ok().flatten()){let _=s.set_item("paint-rust-settings",&data);}}}}
pub fn draft()->Option<Document>{#[cfg(not(target_arch="wasm32"))]{let p=directory().join("draft.json");if std::fs::metadata(&p).ok()?.len()>MAX_BYTES as u64{return None;}Document::from_json(&std::fs::read(p).ok()?).ok()}#[cfg(target_arch="wasm32")]{None}}
pub async fn store_draft(doc:Document,generation:u64)->Result<(),String>{
 if cfg!(test){return Ok(());}static LATEST:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);LATEST.fetch_max(generation,std::sync::atomic::Ordering::SeqCst);
 let data=serde_json::to_vec(&doc).map_err(|e|e.to_string())?;
 #[cfg(not(target_arch="wasm32"))]{static LOCK:std::sync::Mutex<()>=std::sync::Mutex::new(());let _guard=LOCK.lock().map_err(|e|e.to_string())?;if generation<LATEST.load(std::sync::atomic::Ordering::SeqCst){return Ok(());}let dir=directory();std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let temp=dir.join("draft.tmp");std::fs::write(&temp,data).map_err(|e|e.to_string())?;std::fs::rename(temp,dir.join("draft.json")).map_err(|e|e.to_string())?;}
 #[cfg(target_arch="wasm32")]{let db=database().await?;if generation<LATEST.load(std::sync::atomic::Ordering::SeqCst){return Ok(());}let tx=db.transaction(&["drafts"],rexie::TransactionMode::ReadWrite).map_err(|e|e.to_string())?;let store=tx.store("drafts").map_err(|e|e.to_string())?;let value=serde_wasm_bindgen::to_value(&doc).map_err(|e|e.to_string())?;store.put(&value,Some(&wasm_bindgen::JsValue::from_str("current"))).await.map_err(|e|e.to_string())?;tx.done().await.map_err(|e|e.to_string())?;db.close();let _=data;}
 Ok(())
}
#[cfg(target_arch="wasm32")]
async fn database()->Result<rexie::Rexie,String>{rexie::Rexie::builder("openaec-paint").version(1).add_object_store(rexie::ObjectStore::new("drafts")).build().await.map_err(|e|e.to_string())}
#[cfg(target_arch="wasm32")]
pub async fn load_web_draft()->Result<Option<Document>,String>{let db=database().await?;let tx=db.transaction(&["drafts"],rexie::TransactionMode::ReadOnly).map_err(|e|e.to_string())?;let value=tx.store("drafts").map_err(|e|e.to_string())?.get(wasm_bindgen::JsValue::from_str("current")).await.map_err(|e|e.to_string())?;db.close();let Some(value)=value else{return Ok(None)};let doc:Document=serde_wasm_bindgen::from_value(value).map_err(|e|e.to_string())?;doc.validate().map_err(|e|e.to_string())?;Ok(Some(doc))}

pub type OpenResult=Result<Option<(String,Vec<u8>)>,String>;
pub async fn open(ifcx:bool)->OpenResult{let dialog=if ifcx{rfd::AsyncFileDialog::new().add_filter("IFCX raster",&["ifcx","ifcxb"])}else{rfd::AsyncFileDialog::new().add_filter("Afbeeldingen",paint_core::codec::IMAGE_EXTENSIONS)};let Some(file)=dialog.pick_file().await else{return Ok(None)};#[cfg(not(target_arch="wasm32"))]{if std::fs::metadata(file.path()).map_err(|e|e.to_string())?.len()>MAX_BYTES as u64{return Err("Afbeelding is groter dan 64 MB".into());}}#[cfg(target_arch="wasm32")]{if file.inner().size()>MAX_BYTES as f64{return Err("Afbeelding is groter dan 64 MB".into());}}let bytes=file.read().await;if bytes.len()>MAX_BYTES{return Err("Afbeelding is groter dan 64 MB".into());}let name=file.file_name();
 #[cfg(target_arch="wasm32")]let (name,bytes)=if name.to_ascii_lowercase().ends_with(".avif"){let decoded=crate::web::avif_png(&bytes).await?;(format!("{}.png",name.rsplit_once('.').map(|s|s.0).unwrap_or(&name)),decoded)}else{(name,bytes)};
 Ok(Some((name,bytes)))}
pub async fn save(doc:Document,extension:&'static str)->Result<bool,String>{
 let name=format!("{}.{}",doc.name,extension);let Some(file)=rfd::AsyncFileDialog::new().set_file_name(name).add_filter(extension,&[extension]).save_file().await else{return Ok(false)};
 let bytes=paint_core::codec::export_image(&doc,extension).map_err(|e|e.to_string())?;file.write(&bytes).await.map_err(|e|e.to_string())?;Ok(true)
}
pub async fn api(base:String,token:String,id:String,revision:Option<u64>,document:Document,action:&str)->Result<serde_json::Value,String>{
 let parsed=reqwest::Url::parse(&base).map_err(|e|e.to_string())?;if parsed.scheme()!="http"||!matches!(parsed.host_str(),Some("127.0.0.1"|"localhost"|"[::1]"|"::1"))||!parsed.username().is_empty()||parsed.password().is_some(){return Err("Gebruik een lokale HTTP API".into());}
 let client=reqwest::Client::new();let base=base.trim_end_matches('/');let mut request=match action{"list"=>client.get(format!("{base}/v1/documents")),"pull"=>{let id=uuid::Uuid::parse_str(&id).map_err(|e|e.to_string())?;client.get(format!("{base}/v1/documents/{id}"))},"push"=>{if let Some(revision)=revision{let id=uuid::Uuid::parse_str(&id).map_err(|e|e.to_string())?;client.put(format!("{base}/v1/documents/{id}")).json(&serde_json::json!({"document":document,"expectedRevision":revision}))}else{client.post(format!("{base}/v1/documents")).json(&document)}},_=>return Err("Unknown action".into())};request=request.bearer_auth(token);let response=request.send().await.map_err(|e|e.to_string())?;let status=response.status();let value:serde_json::Value=response.json().await.map_err(|e|e.to_string())?;if !status.is_success(){return Err(value["error"].as_str().unwrap_or("API error").into());}Ok(value)
}
