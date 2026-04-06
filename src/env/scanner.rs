use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    let patterns = &*PATTERNS;
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
        "rs",
        "js",
        "ts",
        "jsx",
        "tsx",
        "py",
        "go",
        "rb",
        "java",
        "kt",
        "swift",
        "c",
        "cpp",
        "h",
        "cs",
        "php",
        "ex",
        "exs",
        "erl",
        "hs",
        "ml",
        "yml",
        "yaml",
        "toml",
        "json",
        "Dockerfile",
        "docker-compose.yml",
    ];

    scan_dir(
        project_dir,
        &ignore_dirs,
        &extensions,
        patterns,
        &mut used_vars,
    );

    let used_key_set: std::collections::HashSet<&str> =
        used_vars.keys().map(|s| s.as_str()).collect();
    let defined_key_set: std::collections::HashSet<&str> = defined_keys.iter().copied().collect();

    let unused_vars = defined_keys
        .iter()
        .filter(|k| !used_key_set.contains(*k))
        .map(|k| k.to_string())
        .collect();

    let undefined_vars = used_key_set
        .iter()
        .filter(|k| !defined_key_set.contains(*k))
        .map(|k| k.to_string())
        .collect();

    ScanResult {
        used_vars,
        unused_vars,
        undefined_vars,
    }
}

static PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap(),
        Regex::new(r#"import\.meta\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap(),
        Regex::new(r#"os\.(?:environ\.get|environ\[|getenv)\(?["\']([A-Z_][A-Z0-9_]*)["\']"#)
            .unwrap(),
        Regex::new(r#"env::var\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        Regex::new(r#"os\.Getenv\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        Regex::new(r#"ENV\[?\.?(?:fetch\()?["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        Regex::new(r#"System\.getenv\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        Regex::new(r#"getenv\(["\']([A-Z_][A-Z0-9_]*)["\']"#).unwrap(),
        Regex::new(r#"\$\{([A-Z_][A-Z0-9_]*)\}"#).unwrap(),
        Regex::new(r#"\$([A-Z_][A-Z0-9_]*)\b"#).unwrap(),
    ]
});

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
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !ignore_dirs.contains(&dir_name) && !dir_name.starts_with('.') {
                scan_dir(&path, ignore_dirs, extensions, patterns, results);
            }
            continue;
        }

        if path.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip .env files themselves
            if file_name.starts_with(".env") {
                continue;
            }

            let has_valid_ext = extensions
                .iter()
                .any(|ext| file_name == *ext || file_name.ends_with(&format!(".{}", ext)));

            if has_valid_ext {
                scan_file(&path, patterns, results);
            }
        }
    }
}

fn scan_file(path: &Path, patterns: &[Regex], results: &mut BTreeMap<String, Vec<EnvUsage>>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn match_pattern(input: &str) -> Vec<String> {
        let mut results = Vec::new();
        for pattern in PATTERNS.iter() {
            for cap in pattern.captures_iter(input) {
                if let Some(m) = cap.get(1) {
                    let val = m.as_str().to_string();
                    if !results.contains(&val) {
                        results.push(val);
                    }
                }
            }
        }
        results
    }

    #[test]
    fn test_pattern_js_process_env() {
        assert_eq!(
            match_pattern("const x = process.env.MY_API_KEY"),
            vec!["MY_API_KEY"]
        );
    }

    #[test]
    fn test_pattern_vite_import_meta() {
        assert_eq!(
            match_pattern("import.meta.env.VITE_API_URL"),
            vec!["VITE_API_URL"]
        );
    }

    #[test]
    fn test_pattern_python_getenv() {
        assert_eq!(
            match_pattern("os.getenv('DATABASE_URL')"),
            vec!["DATABASE_URL"]
        );
    }

    #[test]
    fn test_pattern_python_environ_get() {
        assert_eq!(
            match_pattern("os.environ.get('SECRET_KEY')"),
            vec!["SECRET_KEY"]
        );
    }

    #[test]
    fn test_pattern_python_environ_bracket() {
        assert_eq!(match_pattern("os.environ['DB_HOST']"), vec!["DB_HOST"]);
    }

    #[test]
    fn test_pattern_rust_env_var() {
        assert_eq!(
            match_pattern(r#"env::var("API_TOKEN").unwrap()"#),
            vec!["API_TOKEN"]
        );
    }

    #[test]
    fn test_pattern_go_getenv() {
        assert_eq!(match_pattern(r#"os.Getenv("SECRET")"#), vec!["SECRET"]);
    }

    #[test]
    fn test_pattern_ruby_env_bracket() {
        assert_eq!(match_pattern(r#"ENV["REDIS_URL"]"#), vec!["REDIS_URL"]);
    }

    #[test]
    fn test_pattern_java_system_getenv() {
        assert_eq!(
            match_pattern(r#"System.getenv("DB_PASS")"#),
            vec!["DB_PASS"]
        );
    }

    #[test]
    fn test_pattern_php_getenv() {
        assert_eq!(match_pattern(r#"getenv("APP_ENV")"#), vec!["APP_ENV"]);
    }

    #[test]
    fn test_pattern_shell_braces() {
        assert_eq!(match_pattern("echo ${MY_VAR}"), vec!["MY_VAR"]);
    }

    #[test]
    fn test_pattern_shell_dollar() {
        assert_eq!(match_pattern("echo $MY_VAR"), vec!["MY_VAR"]);
    }

    #[test]
    fn test_pattern_no_match() {
        assert!(match_pattern("let x = 42;").is_empty());
    }

    #[test]
    fn test_scan_project_classification() {
        let dir = std::env::temp_dir().join("envi_test_scan");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("app.js"),
            "const a = process.env.USED_KEY;\nconst b = process.env.GHOST_KEY;\n",
        )
        .unwrap();

        let result = scan_project(&dir, &["USED_KEY", "UNUSED_KEY"]);

        assert!(result.used_vars.contains_key("USED_KEY"));
        assert!(result.used_vars.contains_key("GHOST_KEY"));
        assert!(result.unused_vars.contains(&"UNUSED_KEY".to_string()));
        assert!(result.undefined_vars.contains(&"GHOST_KEY".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_ignores_env_files() {
        let dir = std::env::temp_dir().join("envi_test_scan_env");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join(".env"), "SOME_KEY=value\n").unwrap();

        let result = scan_project(&dir, &["SOME_KEY"]);
        assert!(!result.used_vars.contains_key("SOME_KEY"));

        let _ = fs::remove_dir_all(&dir);
    }
}
