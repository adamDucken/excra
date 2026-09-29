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

#[test]
fn procedural_macro_api_changes_report_incomplete_extraction() {
    let workspace = TempDir::new().unwrap();
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\", \"dep\", \"shape\"]\nresolver = \"3\"\n",
    )
    .unwrap();
    for (name, manifest, source) in [
        (
            "app",
            "[dependencies]\ndep = { path = \"../dep\" }\n",
            "pub fn check() { let packet = dep::Packet { byte: 1 }; let _ = packet.byte; dep::S.ghost(); dep::Included.generated(); let _ = dep::Choice::Extra(1); let _: dep::api::Missing; }\n",
        ),
        (
            "dep",
            "[dependencies]\nshape = { path = \"../shape\" }\n",
            "use shape::{packet, method, missing, variants};\n#[packet] pub struct Packet;\n#[method] pub struct S;\n#[variants] pub enum Choice {}\nmissing!();\npub mod nested { #[shape::packet] pub struct Packet; }\npub mod donor { #[shape::packet] pub struct Packet; shape::missing!(); }\npub mod api { pub use crate::donor::*; }\npub struct Included;\ninclude!(concat!(env!(\"OUT_DIR\"), \"/included.rs\"));\npub struct Plain;\n",
        ),
        (
            "shape",
            "[lib]\nproc-macro = true\n",
            "extern crate proc_macro;\nuse proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn packet(_: TokenStream, _: TokenStream) -> TokenStream {\n    \"pub struct Packet { #[cfg(not(doc))] pub byte: u8 }\".parse().unwrap()\n}\n#[proc_macro_attribute]\npub fn method(_: TokenStream, _: TokenStream) -> TokenStream {\n    \"pub struct S; impl S { #[cfg(not(doc))] pub fn ghost(&self) {} }\".parse().unwrap()\n}\n#[proc_macro_attribute]\npub fn variants(_: TokenStream, _: TokenStream) -> TokenStream {\n    \"pub enum Choice { Base, #[cfg(not(doc))] Extra(u8) }\".parse().unwrap()\n}\n#[proc_macro]\npub fn missing(_: TokenStream) -> TokenStream {\n    \"#[cfg(not(doc))] pub struct Missing;\".parse().unwrap()\n}\n",
        ),
    ] {
        let path = workspace.path().join(name);
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(
            path.join("Cargo.toml"),
            format!(
                "[package]\nname = {name:?}\nversion = \"0.1.0\"\nedition = \"2024\"\n{manifest}"
            ),
        )
        .unwrap();
        fs::write(path.join("src/lib.rs"), source).unwrap();
    }
    fs::write(
        workspace.path().join("dep/build.rs"),
        "fn main() { let path = std::path::PathBuf::from(std::env::var_os(\"OUT_DIR\").unwrap()).join(\"included.rs\"); std::fs::write(path, \"impl Included { #[cfg(not(doc))] pub fn generated(&self) {} }\").unwrap(); }\n",
    )
    .unwrap();
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
    let normal = Command::new("cargo")
        .args(["check", "--offline", "--locked", "-p", "app"])
        .current_dir(workspace.path())
        .env("CARGO_TARGET_DIR", workspace.path().join("target"))
        .output()
        .unwrap();
    assert!(
        normal.status.success(),
        "{}",
        String::from_utf8_lossy(&normal.stderr)
    );

    for (name, missing) in [
        ("Packet", "field byte"),
        ("S", "ghost"),
        ("Included", "generated"),
        ("Choice", "variant Extra"),
        ("Missing", "struct"),
        ("nested::Packet", "field byte"),
        ("api::Packet", "field byte"),
    ] {
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
            "{name}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("non-doc API extraction is incomplete:"),
            "{name}: {stderr}"
        );
        if name == "Included" {
            assert!(stderr.contains("included.rs"), "{name}: {stderr}");
        } else {
            assert!(stderr.contains("compiler expansion"), "{name}: {stderr}");
            assert!(stderr.contains(missing), "{name}: {stderr}");
        }
    }
    let missing_glob = Command::new(env!("CARGO_BIN_EXE_excra"))
        .args([
            "use dep::api::Missing;",
            "--root",
            workspace.path().to_str().unwrap(),
            "--package",
            "app",
        ])
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TARGET_DIR", workspace.path().join("target"))
        .output()
        .unwrap();
    assert!(!missing_glob.status.success());
    assert!(
        String::from_utf8_lossy(&missing_glob.stderr)
            .contains("non-doc API extraction is incomplete: compiler accepted import 'dep::api::Missing' but Rustdoc JSON omitted it"),
        "{}",
        String::from_utf8_lossy(&missing_glob.stderr)
    );
    let unaffected = Command::new(env!("CARGO_BIN_EXE_excra"))
        .args([
            "use dep::Plain;",
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
        unaffected.status.success(),
        "{}",
        String::from_utf8_lossy(&unaffected.stderr)
    );
    assert!(String::from_utf8_lossy(&unaffected.stdout).contains("pub struct Plain;"));
}
