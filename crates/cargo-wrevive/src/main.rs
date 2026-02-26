//! # cargo-wrevive
//!
//! Cargo subcommand to build the current project as a PolkaVM contract (RISC-V ELF → .polkavm).
//! 将当前项目构建为 PolkaVM 合约（RISC-V ELF → .polkavm）。
//!
//! Install: `cargo install --path crates/cargo-wrevive`
//! Usage: in a contract project dir, run `cargo wrevive build`

use anyhow::Context;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    print_rust_version();
    let args: Vec<String> = std::env::args().collect();
    // cargo wrevive build ... 时 Cargo 会把 "wrevive" 作为第一个参数传入，需跳过
    // cargo wrevive build ... → Cargo passes "wrevive" as first arg, skip it for subcommand
    // cargo wrevive build 时 Cargo 把 "wrevive" 作为第一个参数传入，需跳过以得到子命令
    let (sub, rest) = match (args.get(1).map(String::as_str), args.get(2).map(String::as_str)) {
        (Some("wrevive"), next) => (next.unwrap_or(""), 3),
        (some_sub, _) => (some_sub.unwrap_or(""), 2),
    };

    match sub {
        "build" => cmd_build(&args[rest..])?,
        "help" | "-h" | "--help" => print_help(),
        "" => {
            print_help();
            anyhow::bail!("missing subcommand");
        }
        _ => {
            eprintln!("unknown subcommand: {sub}");
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn cmd_build(args: &[String]) -> anyhow::Result<()> {
    let mut pkg_name: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-p" || args[i] == "--package" {
            i += 1;
            if i < args.len() {
                pkg_name = Some(args[i].clone());
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    let (project_dir, manifest) = if let Some(name) = pkg_name {
        let manifest = resolve_package_manifest(&name)?;
        let project_dir = manifest.parent().unwrap().to_path_buf();
        (project_dir, manifest)
    } else {
        let project_dir = std::env::current_dir()?;
        let manifest = project_dir.join("Cargo.toml");
        if !manifest.exists() {
            anyhow::bail!("no Cargo.toml in current directory (run from contract project root or use -p <package>)");
        }
        (project_dir, manifest)
    };

    // 若在 workspace 内，则用 workspace 的 target；否则用当前包的 target
    let target_base = get_workspace_or_project_root(&manifest)?;
    let out_dir = target_base
        .join("target")
        .join("release")
        .join("build")
        .join("cargo-wrevive")
        .join("out");
    std::fs::create_dir_all(&out_dir)?;

    unsafe {
        std::env::set_var("OUT_DIR", &out_dir);
        std::env::set_var("CARGO_MANIFEST_DIR", &project_dir);
        std::env::set_var("PROFILE", "release");
    }

    const G: &str = "\x1b[32m";
    const R: &str = "\x1b[0m";
    eprintln!("cargo wrevive build — steps:");
    eprintln!("  {}1/2 build (RISC-V ELF){}", G, R);
    if let Ok(cmd_line) = format_cargo_build_command(&manifest, &target_base) {
        eprintln!("{}", cmd_line);
    }
    eprintln!("  {}2/2 polkavm_linker::program_from_elf (ELF → .polkavm){}", G, R);

    cargo_wrevive::PvmBuilder::new().build();

    Ok(())
}

/// 拼出与 lib build_elf 一致的 cargo build 完整命令（仅用于打印）。
fn format_cargo_build_command(
    manifest: &std::path::Path,
    target_base: &std::path::Path,
) -> anyhow::Result<String> {
    let target_dir = target_base.join("target").join("pvmbuild");
    let mut args = polkavm_linker::TargetJsonArgs::default();
    args.is_64_bit = true;
    let target_json = polkavm_linker::target_json_path(args)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let bins = get_bin_names_from_manifest(manifest)?;
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut parts = vec![
        format!("CARGO_TARGET_DIR={}", target_dir.display()),
        "RUSTFLAGS=\"-Zunstable-options -Cpanic=immediate-abort\"".to_string(),
        "RUSTC_BOOTSTRAP=1".to_string(),
        format!("{} build --manifest-path {} --profile release --target {} -Z build-std=core,alloc -Z json-target-spec",
            cargo,
            manifest.display(),
            target_json.display(),
        ),
    ];
    for b in &bins {
        parts.push(format!("--bin {}", b));
    }
    Ok(parts.join(" "))
}

fn get_bin_names_from_manifest(manifest: &std::path::Path) -> anyhow::Result<Vec<String>> {
    use toml_edit::DocumentMut;
    let content = std::fs::read_to_string(manifest).context("read Cargo.toml")?;
    let doc: DocumentMut = content.parse().context("parse Cargo.toml")?;
    let mut bins = Vec::new();
    if let Some(bin_array) = doc.get("bin").and_then(|b| b.as_array_of_tables()) {
        for bin in bin_array {
            if let Some(name) = bin.get("name").and_then(|n| n.as_str()) {
                bins.push(name.to_string());
            }
        }
    }
    if bins.is_empty() {
        if let Some(name) = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        {
            bins.push(name.to_string());
        }
    }
    Ok(bins)
}

/// 通过 -p 的包名解析出 manifest 路径（从当前目录的 workspace 中查找）。
fn resolve_package_manifest(package_name: &str) -> anyhow::Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    let output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .current_dir(&cwd)
        .output()
        .context("cargo metadata failed")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo metadata failed: {stderr}");
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).context("parse cargo metadata json")?;
    let packages = json
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("cargo metadata missing packages"))?;
    for pkg in packages {
        if pkg.get("name").and_then(|v| v.as_str()) == Some(package_name) {
            let path = pkg
                .get("manifest_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("package missing manifest_path"))?;
            return Ok(std::path::PathBuf::from(path));
        }
    }
    anyhow::bail!("package not found in workspace: {package_name}")
}

/// 通过 cargo metadata 获取 workspace 根目录；非 workspace 则返回 manifest 所在目录。
fn get_workspace_or_project_root(manifest_path: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
    let output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()
        .context("cargo metadata failed")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo metadata failed: {stderr}");
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).context("parse cargo metadata json")?;
    let root = json
        .get("workspace_root")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("cargo metadata missing workspace_root"))?;
    Ok(std::path::PathBuf::from(root))
}

fn print_rust_version() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    match std::process::Command::new(&rustc).arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout);
            eprintln!("{}", v.trim());
        }
        _ => {
            eprintln!("rustc --version: (unable to get version)");
        }
    }
}

fn print_help() {
    eprintln!(
        r#"cargo wrevive — build PolkaVM contract

USAGE:
    cargo wrevive build [OPTIONS]

OPTIONS:
    -p, --package <NAME>     Package to build (workspace member name). Default: current directory.

Run from workspace root and use -p to build a member, or run from the contract directory.
Output: target/<bin>.release.polkavm (under workspace target when in a workspace)
"#
    );
}
