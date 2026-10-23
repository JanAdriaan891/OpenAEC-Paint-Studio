use std::{ffi::OsString, fs::File, io::Read, path::PathBuf};

pub fn launch_paths(arguments: impl Iterator<Item = OsString>) -> Vec<PathBuf> {
    arguments.filter(|value| !value.to_string_lossy().starts_with('-'))
        .map(PathBuf::from).filter(|path| path.is_file()).collect()
}

pub fn read_launch_file(path: &PathBuf) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > 64_000_000 {
        return Err("Only files up to 64 MB can be opened".into());
    }
    let mut bytes = Vec::new();
    file.take(64_000_001).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    if bytes.len() > 64_000_000 { return Err("File exceeds 64 MB".into()); }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launch_accepts_real_paths_with_spaces_and_ignores_flags() {
        let path = std::env::temp_dir().join(format!("paint launch {}.png", std::process::id()));
        std::fs::write(&path, b"image bytes").unwrap();
        let paths = launch_paths(vec![OsString::from("--flag"), OsString::from("/nonexistent-paint-file"), path.clone().into_os_string()].into_iter());
        assert_eq!(paths, vec![path.clone()]);
        assert_eq!(read_launch_file(&path).unwrap(), b"image bytes");
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn oversized_launch_file_is_rejected_before_reading() {
        let path = std::env::temp_dir().join(format!("paint oversized {}.png", std::process::id()));
        File::create(&path).unwrap().set_len(64_000_001).unwrap();
        assert!(read_launch_file(&path).unwrap_err().contains("64 MB"));
        std::fs::remove_file(path).unwrap();
    }
}
