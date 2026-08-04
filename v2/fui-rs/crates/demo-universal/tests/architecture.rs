use std::fs;
use std::path::Path;

fn rust_sources(path: &Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(path).expect("read universal demo source directory") {
        let entry = entry.expect("read universal demo source entry");
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(fs::read_to_string(path).expect("read universal demo Rust source"));
        }
    }
}

#[test]
fn universal_modules_do_not_import_shell_or_platform_implementation_details() {
    let mut sources = Vec::new();
    rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
    );
    let source = sources.join("\n");
    for forbidden in [
        "current_route(",
        "navigation::",
        "NavLink",
        "target_os",
        "extern \"C\"",
        "browser-bridge",
        "native/macos",
        "native/windows",
        "native/linux",
    ] {
        assert!(
            !source.contains(forbidden),
            "universal demo source contains shell/platform detail {forbidden}"
        );
    }
}
