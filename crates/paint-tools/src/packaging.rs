use crate::{copy, run, tree, VERSION};
use anyhow::{ensure, Context, Result};
use std::{fs, path::Path, process::Command};

/// Notices are gathered from the exact locked dependencies, including target-specific crates.
pub fn legal(out: &Path) -> Result<()> {
    fs::create_dir_all(out)?;
    for p in [
        "LICENSE",
        "THIRD_PARTY_NOTICES.md",
        "vendor/BLUEPRINT-REVISION",
        "release-public.pem",
    ] {
        copy(p, out.join(Path::new(p).file_name().unwrap()))?;
    }
    copy(
        "vendor/openaec-ui/LICENSE.md",
        out.join("Blueprint-LGPL.md"),
    )?;
    tree(
        Path::new("vendor/openaec-ui/licenses"),
        &out.join("Blueprint-assets"),
    )?;
    copy(
        "public/brand/LICENSE",
        out.join("OpenAEC-assets-CC-BY-SA-4.0.txt"),
    )?;
    copy(
        "assets/fonts/DejaVu-LICENSE.txt",
        out.join("DejaVu-fonts.txt"),
    )?;
    let metadata = Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
        .output()?;
    ensure!(
        metadata.status.success(),
        "cargo metadata: {}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout)?;
    let packages = metadata["packages"].as_array().unwrap();
    let mut list = Vec::new();
    for package in packages {
        let name = package["name"].as_str().unwrap();
        let version = package["version"].as_str().unwrap();
        let dir = Path::new(package["manifest_path"].as_str().unwrap())
            .parent()
            .unwrap();
        let destination = out.join("crates").join(format!("{name}-{version}"));
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let label = entry.file_name().to_string_lossy().to_lowercase();
            if ["license", "licence", "copying", "notice", "copyright"]
                .iter()
                .any(|s| label.starts_with(s))
            {
                if entry.file_type()?.is_dir() {
                    tree(&entry.path(), &destination.join(entry.file_name()))?;
                } else {
                    copy(entry.path(), destination.join(entry.file_name()))?;
                }
            }
        }
        list.push(serde_json::json!({"name":name,"version":version,"license":package["license"],"source":package["source"],"repository":package["repository"]}));
    }
    fs::write(
        out.join("dependencies.json"),
        serde_json::to_vec_pretty(&list)?,
    )?;
    fs::write(out.join("SOURCE.txt"),format!("Complete corresponding source and build instructions: https://github.com/JanAdriaan891/OpenAEC-Paint-Studio/releases/tag/v{VERSION}\nDownload the -source.tar.gz asset for the application, vendored dependencies and offline Cargo configuration. Rebuild and replace the executables to use modified LGPL components. There is no installation key or signature lock.\n"))?;
    Ok(())
}

pub fn services(out: &Path, source: &Path, label: &str) -> Result<()> {
    fs::create_dir_all(out)?;
    let root = out.join(format!("services-{label}"));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    for bin in ["paint-api", "paint-mcp"] {
        let name = format!("{bin}{}", std::env::consts::EXE_SUFFIX);
        copy(source.join(&name), root.join(name))?;
    }
    legal(&root.join("licenses"))?;
    copy("docs/API.md", root.join("API.md"))?;
    copy("docs/MCP.md", root.join("MCP.md"))?;
    let archive = out.join(format!(
        "OpenAEC-Paint-Studio_{VERSION}_{label}_services.tar.gz"
    ));
    run(Command::new("tar")
        .arg("-czf")
        .arg(archive)
        .arg("-C")
        .arg(root)
        .arg("."))
}

pub fn windows(out: &Path) -> Result<()> {
    fs::create_dir_all(out)?;
    let out = canonical(out)?;
    let source = canonical(Path::new("target/release"))?;
    let licenses = out.join("licenses");
    legal(&licenses).context("Collect Windows dependency licenses")?;
    let installer = out.join(format!("OpenAEC-Paint-Studio_{VERSION}_x64-setup.exe"));
    // NSIS uses the script's directory for relative icon paths.
    let compiler = std::env::var_os("MAKENSIS").unwrap_or_else(|| "makensis".into());
    run(Command::new(compiler)
        .arg(format!("/DNUMBER={VERSION}"))
        .arg(format!("/DSOURCE={}", source.display()))
        .arg(format!("/DLEGAL={}", licenses.display()))
        .arg(format!("/DOUTPUT={}", installer.display()))
        .arg("packaging/windows/installer.nsi"))
    .context("Run NSIS: install NSIS and set MAKENSIS to makensis.exe")
}

fn canonical(path: &Path) -> Result<std::path::PathBuf> {
    let path = path.canonicalize()?;
    #[cfg(windows)]
    {
        let value = path.to_string_lossy();
        if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
            return Ok(format!(r"\\{rest}").into());
        }
        if let Some(rest) = value.strip_prefix(r"\\?\") {
            return Ok(rest.into());
        }
    }
    Ok(path)
}

pub fn macos(out: &Path, source: &Path) -> Result<()> {
    fs::create_dir_all(out)?;
    let stage = out.join("macos-stage");
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    let app = stage.join("OpenAEC Paint Studio.app");
    let contents = app.join("Contents");
    for bin in ["openaec-paint-studio", "paint-api", "paint-mcp"] {
        copy(source.join(bin), contents.join("MacOS").join(bin))?;
    }
    let plist = fs::read_to_string("packaging/macos/Info.plist.in")?.replace("@VERSION@", VERSION);
    fs::write(contents.join("Info.plist"), plist)?;
    copy(
        "packaging/icons/icon.icns",
        contents.join("Resources/icon.icns"),
    )?;
    legal(&contents.join("Resources/licenses"))?;
    let identity = std::env::var("APPLE_SIGNING_IDENTITY").unwrap_or_else(|_| "-".into());
    for bin in ["paint-api", "paint-mcp"] {
        run(Command::new("codesign")
            .args(["--force", "--sign", &identity, "--timestamp=none"])
            .arg(contents.join("MacOS").join(bin)))?;
    }
    run(Command::new("codesign")
        .args(["--force", "--sign", &identity, "--timestamp=none"])
        .arg(&app))?;
    run(Command::new("codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(&app))?;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/Applications", stage.join("Applications"))?;
    }
    let dmg = out.join(format!("OpenAEC-Paint-Studio_{VERSION}_universal.dmg"));
    run(Command::new("hdiutil")
        .args([
            "create",
            "-volname",
            "OpenAEC Paint Studio",
            "-format",
            "UDZO",
            "-ov",
            "-srcfolder",
        ])
        .arg(&stage)
        .arg(&dmg))?;
    run(Command::new("hdiutil").arg("verify").arg(dmg))
}
