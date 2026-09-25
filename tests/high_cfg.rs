use std::{fs, process::Command};
use tempfile::TempDir;

#[test]
fn non_doc_only_api_reports_incomplete_extraction() {
    let workspace = TempDir::new().unwrap();
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\", \"dep\"]\nresolver = \"3\"\n",
    )
    .unwrap();
    for (name, dependencies, source) in [
        ("app", "[dependencies]\ndep = { path = \"../dep\" }\n", ""),
        (
            "dep",
            "",
            "pub struct Packet { #[cfg(not(doc))] pub byte: u8 }\n#[cfg(doc)] pub struct Choice { pub docs: u8 }\n#[cfg(not(doc))] pub struct Choice { pub actual: u8 }\npub struct Unrelated;\n",
        ),
    ] {
        let path = workspace.path().join(name);
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(
            path.join("Cargo.toml"),
            format!("[package]\nname = {name:?}\nversion = \"0.1.0\"\nedition = \"2024\"\n{dependencies}"),
        )
        .unwrap();
        fs::write(path.join("src/lib.rs"), source).unwrap();
    }
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(workspace.path())
        .output()
        .unwrap();
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );

    for name in ["Packet", "Choice"] {
        let output = Command::new(env!("CARGO_BIN_EXE_excra"))
            .args([
                format!("use dep::{name};"),
                "--root".into(),
                workspace.path().display().to_string(),
                "--package".into(),
                "app".into(),
            ])
            .env("CARGO_NET_OFFLINE", "true")
            .env("CARGO_TARGET_DIR", workspace.path().join("target"))
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("non-doc API extraction is incomplete"),
            "{name}: {stderr}"
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_excra"))
        .args([
            "use dep::Unrelated;",
            "--root",
            workspace.path().to_str().unwrap(),
            "--package",
            "app",
        ])
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TARGET_DIR", workspace.path().join("target"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("definition: pub struct Unrelated;"));

    fs::write(workspace.path().join("dep/src/lib.rs"), "pub mod nested;\n").unwrap();
    fs::write(
        workspace.path().join("dep/src/nested.rs"),
        "pub struct Packet { #[cfg(not(doc))] pub byte: u8 }\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_excra"))
        .args([
            "use dep::nested::Packet;",
            "--root",
            workspace.path().to_str().unwrap(),
            "--package",
            "app",
        ])
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TARGET_DIR", workspace.path().join("target"))
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("non-doc API extraction is incomplete"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::write(
        workspace.path().join("dep/src/lib.rs"),
        "macro_rules! packet { () => { pub struct Packet { #[cfg(not(doc))] pub byte: u8 } } }\npacket!();\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_excra"))
        .args([
            "use dep::Packet;",
            "--root",
            workspace.path().to_str().unwrap(),
            "--package",
            "app",
        ])
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TARGET_DIR", workspace.path().join("target"))
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("non-doc API extraction is incomplete"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::write(
        workspace.path().join("dep/src/lib.rs"),
        "pub struct Packet;\nimpl Packet { #[cfg(not(doc))] pub fn byte(&self) -> u8 { 0 } }\npub enum Choice { #[cfg(not(doc))] Actual, Docs }\n",
    )
    .unwrap();
    for name in ["Packet", "Choice"] {
        let output = Command::new(env!("CARGO_BIN_EXE_excra"))
            .args([
                format!("use dep::{name};"),
                "--root".into(),
                workspace.path().display().to_string(),
                "--package".into(),
                "app".into(),
            ])
            .env("CARGO_NET_OFFLINE", "true")
            .env("CARGO_TARGET_DIR", workspace.path().join("target"))
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("non-doc API extraction is incomplete"),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
