use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EnvUsage {
    pub file: PathBuf,
    pub line: usize,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Env vars found in code, with their usage locations
    pub used_vars: BTreeMap<String, Vec<EnvUsage>>,
    /// Vars defined in .env but never used in code
    pub unused_vars: Vec<String>,
    /// Vars used in code but not defined in any .env
    pub undefined_vars: Vec<String>,
}

pub fn scan_project(project_dir: &Path, defined_keys: &[&str]) -> ScanResult {
    let patterns = build_patterns();
    let mut used_vars: BTreeMap<String, Vec<EnvUsage>> = BTreeMap::new();

    let ignore_dirs = [
        "node_modules",
        ".git",
        "target",
        "dist",
        "build",
        ".next",
        "__pycache__",
        "vendor",
        ".venv",
        "venv",
    ];

    let extensions = [
        "rs", "js", "ts", "jsx", "tsx", "py", "go", "rb", "java", "kt", "swift", "c", "cpp",
        "h", "cs", "php", "ex", "exs", "erl", "hs", "ml", "yml", "yaml", "toml", "json",
        "Dockerfile", "docker-compose.yml",
    ];

    scan_dir(
        project_dir,
        &ignore_dirs,
        &extensions,
        &patterns,
        &mut used_vars,
    );

    let used_keys: Vec<&str> = used_vars.keys().map(|s| s.as_str()).collect();

    let unused_vars = defined_keys
        .iter()
        .filter(|k| !used_keys.contains(k))
        .map(|k| k.to_string())
        .collect();

    let undefined_vars = used_keys
        .iter()
        .filter(|k| !defined_keys.contains(k))
        .map(|k| k.to_string())
        .collect();

    ScanResult {
        used_vars,
        unused_vars,
        undefined_vars,
    }
}

fn build_patterns() -> Vec<Regex> {
    vec![
        // process.env.VAR_NAME (JS/TS)
        Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap(),
        // import.meta.env.VAR_NAME (Vite)
        Regex::new(r#"import\.meta\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap(),
        // os.environ["VAR"] or os.environ.get("VAR") or os.getenv("VAR") (Python)
        Regex::new(r#"os\.(?:environ\.get|environ\[|getenv)\(?["\']([A-Z_][A-Z0-9_]*)["\']"#)
            .unwrap(),
        // env::var("VAR") or std::env::var("VAR") (Rust)
        Regex::new(r#"env::var\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        // os.Getenv("VAR") (Go)
        Regex::new(r#"os\.Getenv\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        // ENV["VAR"] or ENV.fetch("VAR") (Ruby)
        Regex::new(r#"ENV\[?\.?(?:fetch\()?["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        // System.getenv("VAR") (Java/Kotlin)
        Regex::new(r#"System\.getenv\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        // getenv("VAR") (PHP/C)
        Regex::new(r#"getenv\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        // ${VAR_NAME} in YAML/Docker/shell
        Regex::new(r#"\$\{([A-Z_][A-Z0-9_]*)\}"#).unwrap(),
        // $VAR_NAME in shell/Docker (but not $$ or $0-$9)
        Regex::new(r#"\$([A-Z_][A-Z0-9_]*)\b"#).unwrap(),
    ]
}

fn scan_dir(
    dir: &Path,
    ignore_dirs: &[&str],
    extensions: &[&str],
    patterns: &[Regex],
    results: &mut BTreeMap<String, Vec<EnvUsage>>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !ignore_dirs.contains(&dir_name) && !dir_name.starts_with('.') {
                scan_dir(&path, ignore_dirs, extensions, patterns, results);
            }
            continue;
        }

        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Skip .env files themselves
            if file_name.starts_with(".env") {
                continue;
            }

            let has_valid_ext = extensions.iter().any(|ext| {
                file_name == *ext || file_name.ends_with(&format!(".{}", ext))
            });

            if has_valid_ext {
                scan_file(&path, patterns, results);
            }
        }
    }
}

fn scan_file(
    path: &Path,
    patterns: &[Regex],
    results: &mut BTreeMap<String, Vec<EnvUsage>>,
) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for (line_idx, line) in content.lines().enumerate() {
        for pattern in patterns {
            for cap in pattern.captures_iter(line) {
                if let Some(var_match) = cap.get(1) {
                    let var_name = var_match.as_str().to_string();
                    let usage = EnvUsage {
                        file: path.to_path_buf(),
                        line: line_idx + 1,
                        context: line.trim().to_string(),
                    };
                    results.entry(var_name).or_default().push(usage);
                }
            }
        }
    }
}
