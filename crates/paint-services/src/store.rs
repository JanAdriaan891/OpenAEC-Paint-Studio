use anyhow::{bail, ensure, Result};
use paint_core::{Document, Operation, MAX_BYTES};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stored {
    pub id: Uuid,
    pub revision: u64,
    pub updated_at: String,
    pub document: Document,
}
pub struct Store {
    directory: PathBuf,
}
impl Store {
    pub fn new(directory: impl Into<PathBuf>) -> Result<Self> {
        let directory = directory.into();
        fs::create_dir_all(&directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
        }
        Ok(Self { directory })
    }
    fn path(&self, id: Uuid) -> PathBuf {
        self.directory.join(format!("{id}.json"))
    }
    pub fn get(&self, id: Uuid) -> Result<Stored> {
        let path = self.path(id);
        ensure!(
            fs::metadata(&path)?.len() <= MAX_BYTES as u64,
            "Stored document exceeds 64 MB"
        );
        let doc: Stored = serde_json::from_slice(&fs::read(path)?)?;
        doc.document.validate()?;
        ensure!(doc.id == id && doc.revision > 0, "Invalid stored document");
        Ok(doc)
    }
    pub fn list(&self) -> Result<serde_json::Value> {
        let mut out = vec![];
        for entry in fs::read_dir(&self.directory)? {
            let path = entry?.path();
            let Some(id) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| Uuid::parse_str(s).ok())
            else {
                continue;
            };
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let d = self.get(id)?;
            out.push(serde_json::json!({"id":id,"revision":d.revision,"updatedAt":d.updated_at,"name":d.document.name,"width":d.document.width,"height":d.document.height}));
        }
        Ok(out.into())
    }
    fn write(&self, doc: Stored) -> Result<Stored> {
        doc.document.validate()?;
        let path = self.path(doc.id);
        let temp = self.directory.join(format!(".{}.tmp", Uuid::new_v4()));
        write_private(&temp, &serde_json::to_vec(&doc)?)?;
        fs::rename(temp, path)?;
        Ok(doc)
    }
    pub fn create(&self, document: Document) -> Result<Stored> {
        ensure!(
            self.list()?.as_array().unwrap().len() < 100,
            "Workspace limit reached (100 documents)"
        );
        self.write(Stored {
            id: Uuid::new_v4(),
            revision: 1,
            updated_at: timestamp(),
            document,
        })
    }
    pub fn replace(&self, id: Uuid, document: Document, expected: u64) -> Result<Stored> {
        let old = self.get(id)?;
        if old.revision != expected {
            bail!("REVISION_CONFLICT");
        }
        self.write(Stored {
            id,
            revision: old.revision + 1,
            updated_at: timestamp(),
            document,
        })
    }
    pub fn append(&self, id: Uuid, ops: Vec<Operation>, expected: u64) -> Result<Stored> {
        ensure!(
            !ops.is_empty() && ops.len() <= 200,
            "Append needs 1–200 operations"
        );
        let mut old = self.get(id)?;
        old.document.operations.extend(ops);
        self.replace(id, old.document, expected)
    }
    pub fn undo(&self, id: Uuid, expected: u64) -> Result<Stored> {
        let mut old = self.get(id)?;
        old.document.operations.pop();
        self.replace(id, old.document, expected)
    }
}
fn timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
pub fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut opts = fs::OpenOptions::new();
    opts.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
pub fn data_directory() -> PathBuf {
    std::env::var_os("PAINT_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".paint-data"))
}
pub fn api_token(directory: &Path) -> Result<String> {
    if let Ok(token) = std::env::var("PAINT_API_TOKEN") {
        ensure!(!token.is_empty(), "Empty API token");
        return Ok(token);
    }
    fs::create_dir_all(directory)?;
    let path = directory.join("token");
    if path.exists() {
        return Ok(fs::read_to_string(path)?.trim().to_owned());
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    write_private(&path, token.as_bytes())?;
    Ok(token)
}
