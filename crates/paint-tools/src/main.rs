use anyhow::{anyhow, ensure, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{
    pkcs8::{spki::der::pem::LineEnding, DecodePrivateKey, DecodePublicKey, EncodePublicKey},
    Signature, Signer, SigningKey, Verifier, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
mod packaging;
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Serialize, Deserialize)]
struct Manifest {
    schema: u8,
    file: String,
    bytes: u64,
    sha256: String,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn name(path: &Path) -> Result<String> {
    Ok(path
        .file_name()
        .ok_or_else(|| anyhow!("Missing filename"))?
        .to_string_lossy()
        .into_owned())
}
fn adjacent(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", path.display()))
}
fn sign(path: &Path, key_path: &Path) -> Result<()> {
    let key = SigningKey::from_pkcs8_pem(&fs::read_to_string(key_path)?)?;
    let data = fs::read(path)?;
    let manifest = Manifest {
        schema: 1,
        file: name(path)?,
        bytes: data.len() as u64,
        sha256: digest(&data),
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    fs::write(adjacent(path, ".manifest.json"), &bytes)?;
    fs::write(
        adjacent(path, ".sig"),
        format!("{}\n", STANDARD.encode(key.sign(&bytes).to_bytes())),
    )?;
    fs::write(
        adjacent(path, ".pub.pem"),
        key.verifying_key().to_public_key_pem(LineEnding::LF)?,
    )?;
    println!("Signed {}", name(path)?);
    Ok(())
}
fn verify(path: &Path, public: &Path) -> Result<()> {
    let key = VerifyingKey::from_public_key_pem(&fs::read_to_string(public)?)?;
    let bytes = fs::read(adjacent(path, ".manifest.json"))?;
    let signature = Signature::from_slice(
        &STANDARD.decode(fs::read_to_string(adjacent(path, ".sig"))?.trim())?,
    )?;
    key.verify(&bytes, &signature)?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    let artifact = fs::read(path)?;
    ensure!(
        manifest.schema == 1
            && manifest.file == name(path)?
            && manifest.bytes == artifact.len() as u64
            && manifest.sha256 == digest(&artifact),
        "Artifact does not match signed manifest"
    );
    println!("Verified {}", name(path)?);
    Ok(())
}
fn checksums(dir: &Path) -> Result<()> {
    let mut files = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let file = p.file_name().unwrap().to_string_lossy();
            p.is_file()
                && file != "SHA256SUMS"
                && file != "release-public.pem"
                && ![".sig", ".manifest.json", ".pub.pem"]
                    .iter()
                    .any(|suffix| file.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    files.sort();
    let mut out = String::new();
    for p in files {
        out += &format!("{}  {}\n", digest(&fs::read(&p)?), name(&p)?);
    }
    fs::write(dir.join("SHA256SUMS"), out)?;
    Ok(())
}
fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    ensure!(status.success(), "Command failed: {command:?}");
    Ok(())
}
fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = dst.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}
fn tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        if e.file_type()?.is_dir() {
            tree(&e.path(), &dst.join(e.file_name()))?;
        } else {
            copy(e.path(), dst.join(e.file_name()))?;
        }
    }
    Ok(())
}
fn package_linux(out: &Path) -> Result<()> {
    fs::create_dir_all(out)?;
    let root = out.join("deb-root");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(root.join("DEBIAN"))?;
    for bin in ["openaec-paint-studio", "paint-api", "paint-mcp"] {
        copy(
            Path::new("target/release").join(bin),
            root.join("usr/bin").join(bin),
        )?;
    }
    copy(
        "packaging/linux/org.openaec.paint-studio.desktop",
        root.join("usr/share/applications/org.openaec.paint-studio.desktop"),
    )?;
    copy(
        "packaging/linux/openaec-paint-studio.xml",
        root.join("usr/share/mime/packages/openaec-paint-studio.xml"),
    )?;
    copy(
        "packaging/icons/128x128.png",
        root.join("usr/share/icons/hicolor/128x128/apps/org.openaec.paint-studio.png"),
    )?;
    packaging::legal(&root.join("usr/share/doc/open-aec-paint-studio"))?;
    fs::write(root.join("DEBIAN/control"),format!("Package: open-aec-paint-studio\nVersion: {VERSION}\nArchitecture: amd64\nMaintainer: OpenAEC contributors\nSection: graphics\nPriority: optional\nDepends: libxkbcommon0, libx11-6, libxcb1, libwayland-client0\nDescription: OpenAEC Paint Studio native Rust image editor\n"))?;
    run(Command::new("dpkg-deb")
        .arg("--build")
        .arg("--root-owner-group")
        .arg(&root)
        .arg(out.join(format!("OpenAEC-Paint-Studio_{VERSION}_amd64.deb"))))?;
    let app = out.join("OpenAEC Paint Studio.AppDir");
    if app.exists() {
        fs::remove_dir_all(&app)?;
    }
    tree(&root.join("usr"), &app.join("usr"))?;
    copy(
        "packaging/linux/org.openaec.paint-studio.desktop",
        app.join("org.openaec.paint-studio.desktop"),
    )?;
    copy(
        "packaging/icons/128x128.png",
        app.join("org.openaec.paint-studio.png"),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let apprun = app.join("AppRun");
        fs::write(&apprun,"#!/bin/sh\nAPPDIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\nexport LD_LIBRARY_PATH=\"$APPDIR/usr/lib:$APPDIR/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}\"\nexec \"$APPDIR/usr/bin/openaec-paint-studio\" \"$@\"\n")?;
        fs::set_permissions(apprun, fs::Permissions::from_mode(0o755))?;
    }
    println!("DEB and AppDir ready in {}", out.display());
    Ok(())
}
fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let action = args.first().and_then(|s| s.to_str()).unwrap_or("");
    let arg = |i: usize| {
        args.get(i)
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("Missing argument {i}"))
    };
    match action{
 "sign"=>{let key=args.get(2).map(PathBuf::from).or_else(||std::env::var_os("PAINT_SIGNING_KEY_FILE").map(PathBuf::from)).ok_or_else(||anyhow!("Set PAINT_SIGNING_KEY_FILE or supply key path"))?;sign(&arg(1)?,&key)},
 "verify"=>verify(&arg(1)?,&arg(2)?),"checksums"=>checksums(&arg(1)?),"package-linux"=>package_linux(&arg(1)?),
 "legal"=>packaging::legal(&arg(1)?),
 "package-windows"=>packaging::windows(&arg(1)?),
 "package-macos"=>packaging::macos(&arg(1)?,&arg(2)?),
 "package-services"=>packaging::services(&arg(1)?,&arg(2)?,args.get(3).and_then(|s|s.to_str()).ok_or_else(||anyhow!("Missing platform label"))?),
 _=>anyhow::bail!("Usage: paint-release sign <artifact> [key.pem] | verify <artifact> <trusted-public.pem> | checksums <directory> | package-linux <output>")
}
}
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::pkcs8::EncodePrivateKey;
    #[test]
    fn signatures_reject_tampering_and_wrong_keys() {
        let dir = tempfile::tempdir().unwrap();
        let key = SigningKey::from_bytes(&[7; 32]);
        let private = dir.path().join("private.pem");
        fs::write(
            &private,
            key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
        )
        .unwrap();
        let p = dir.path().join("image.bin");
        fs::write(&p, b"image pixels").unwrap();
        sign(&p, &private).unwrap();
        let public = adjacent(&p, ".pub.pem");
        verify(&p, &public).unwrap();
        fs::write(&p, b"changed").unwrap();
        assert!(verify(&p, &public).is_err());
        fs::write(&p, b"image pixels").unwrap();
        fs::write(
            &public,
            SigningKey::from_bytes(&[8; 32])
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        )
        .unwrap();
        assert!(verify(&p, &public).is_err());
    }
}
