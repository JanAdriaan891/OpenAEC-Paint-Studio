//! Portable, lossless project envelope. The API's v1 Document remains unchanged.
use crate::{Document, MAX_BYTES};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Project {
    format: String,
    version: u32,
    document: Document,
}
pub fn write(document: &Document) -> Result<Vec<u8>> {
    document.validate()?;
    let bytes = serde_json::to_vec(&Project {
        format: "openaec-paint-project".into(),
        version: 1,
        document: document.clone(),
    })?;
    ensure!(bytes.len() <= MAX_BYTES, "Project exceeds 64 MB");
    Ok(bytes)
}
pub fn read(bytes: &[u8]) -> Result<Document> {
    ensure!(bytes.len() <= MAX_BYTES, "Project exceeds 64 MB");
    let project: Project = serde_json::from_slice(bytes)?;
    ensure!(
        project.format == "openaec-paint-project" && project.version == 1,
        "Unsupported Paint project version"
    );
    project.document.validate()?;
    // Verify embedded images and rendering before replacing the user's document.
    crate::render(&project.document)?;
    Ok(project.document)
}
