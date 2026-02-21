use std::error::Error;
use std::path::{Path, PathBuf};

pub fn scan_files(path: &Path, recursive: bool) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }

    let mut out = Vec::new();
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(out);
    }
    if recursive {
        let mut stack = vec![path.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let p = entry.path();
                if p.is_file() {
                    out.push(p);
                } else if p.is_dir() {
                    stack.push(p);
                }
            }
        }
    } else {
        for entry in std::fs::read_dir(path)? {
            let p = entry?.path();
            if p.is_file() {
                out.push(p);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_clear_error_for_missing_path() {
        let missing = PathBuf::from("/definitely/not/existing/mybot-path");
        let err = scan_files(&missing, false).expect_err("must fail");
        assert!(err.to_string().contains("Path does not exist"));
    }
}
