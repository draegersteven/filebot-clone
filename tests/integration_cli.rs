use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn tempdir() -> PathBuf {
    let mut p = std::env::temp_dir();
    let uniq = format!("mybot-test-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    p.push(uniq);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn plan_and_apply_dry_run() {
    let dir = tempdir();
    let movie = dir.join("Der.Teufel.traegt.Prada.2006.German.AC3.DL.1080p.BluRay.x265-FuN.mkv");
    let unknown = dir.join("notes.txt");
    fs::write(&movie, b"data").unwrap();
    fs::write(&unknown, b"data").unwrap();

    let out1 = Command::new("cargo")
        .args(["run", "--quiet", "--", "plan", dir.to_str().unwrap(), "--format", "{title} ({year}).{ext}", "--dry-run"])
        .output()
        .unwrap();
    assert!(out1.status.success());
    let s1 = String::from_utf8_lossy(&out1.stdout);
    assert!(s1.contains("PLAN Rename"));
    assert!(s1.contains("SKIP unknown"));

    let out2 = Command::new("cargo")
        .args(["run", "--quiet", "--", "apply", dir.to_str().unwrap(), "--format", "{title} ({year}).{ext}", "--dry-run"])
        .output()
        .unwrap();
    assert!(out2.status.success());
    assert!(movie.exists());

    let _ = fs::remove_dir_all(&dir);
}
