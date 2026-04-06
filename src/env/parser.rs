use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteStyle {
    None,
    Double,
    Single,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
    pub line_number: usize,
    pub is_encrypted: bool,
    pub has_export: bool,
    pub quote_style: QuoteStyle,
}

#[derive(Debug, Clone)]
pub struct EnvFile {
    pub path: PathBuf,
    pub entries: Vec<EnvEntry>,
    pub errors: Vec<(usize, String)>,
}

impl EnvFile {
    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    pub fn get(&self, key: &str) -> Option<&EnvEntry> {
        self.entries.iter().find(|e| e.key == key)
    }

    #[allow(dead_code)]
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.key.as_str()).collect()
    }

    pub fn as_map(&self) -> BTreeMap<&str, &EnvEntry> {
        self.entries.iter().map(|e| (e.key.as_str(), e)).collect()
    }
}

pub fn parse_file(path: &Path) -> Result<EnvFile, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(parse_content(&content, path.to_path_buf()))
}

pub fn parse_content(content: &str, path: PathBuf) -> EnvFile {
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    let mut pending_comment: Option<String> = None;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            pending_comment = None;
            continue;
        }

        if trimmed.starts_with('#') {
            let comment_text = trimmed.trim_start_matches('#').trim().to_string();
            pending_comment = Some(comment_text);
            continue;
        }

        // Remove optional "export " prefix
        let has_export = trimmed.starts_with("export ");
        let line_content = if has_export {
            &trimmed["export ".len()..]
        } else {
            trimmed
        };

        match parse_line(line_content, line_number) {
            Ok(mut entry) => {
                entry.comment = pending_comment.take();
                entry.has_export = has_export;
                entries.push(entry);
            }
            Err(msg) => {
                errors.push((line_number, msg));
                pending_comment = None;
            }
        }
    }

    EnvFile {
        path,
        entries,
        errors,
    }
}

fn parse_line(line: &str, line_number: usize) -> Result<EnvEntry, String> {
    let eq_pos = line
        .find('=')
        .ok_or_else(|| format!("No '=' found on line {}", line_number))?;

    let key = line[..eq_pos].trim().to_string();
    if key.is_empty() {
        return Err(format!("Empty key on line {}", line_number));
    }

    let raw_value = line[eq_pos + 1..].trim();
    let (value, is_encrypted, quote_style) = parse_value(raw_value);

    Ok(EnvEntry {
        key,
        value,
        comment: None,
        line_number,
        is_encrypted,
        has_export: false,
        quote_style,
    })
}

fn parse_value(raw: &str) -> (String, bool, QuoteStyle) {
    let is_encrypted = raw.starts_with("ENC[") && raw.ends_with(']');

    if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
        let value = raw[1..raw.len() - 1].to_string();
        return (value, is_encrypted, QuoteStyle::Double);
    }

    if raw.starts_with('\'') && raw.ends_with('\'') && raw.len() >= 2 {
        let value = raw[1..raw.len() - 1].to_string();
        return (value, is_encrypted, QuoteStyle::Single);
    }

    let value = raw.split(" #").next().unwrap_or(raw).trim().to_string();
    (value, is_encrypted, QuoteStyle::None)
}

pub fn discover_env_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let patterns = [
        ".env",
        ".env.local",
        ".env.development",
        ".env.dev",
        ".env.staging",
        ".env.production",
        ".env.prod",
        ".env.test",
        ".env.example",
        ".env.sample",
        ".env.template",
    ];

    for pattern in &patterns {
        let path = dir.join(pattern);
        if path.exists() && path.is_file() {
            files.push(path);
        }
    }

    // Also check for any .env.* files we might have missed
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(".env") && path.is_file() && !files.contains(&path) {
                    files.push(path);
                }
            }
        }
    }

    files.sort_by(|a, b| {
        let a_name = a.file_name().unwrap_or_default().to_string_lossy();
        let b_name = b.file_name().unwrap_or_default().to_string_lossy();
        // .env first, then .env.example, then alphabetical
        match (a_name.as_ref(), b_name.as_ref()) {
            (".env", _) => std::cmp::Ordering::Less,
            (_, ".env") => std::cmp::Ordering::Greater,
            (".env.example", _) => std::cmp::Ordering::Less,
            (_, ".env.example") => std::cmp::Ordering::Greater,
            _ => a_name.cmp(&b_name),
        }
    });

    files
}

pub fn write_env_file(env_file: &EnvFile) -> Result<(), String> {
    let mut output = String::new();

    for entry in &env_file.entries {
        if let Some(comment) = &entry.comment {
            output.push_str(&format!("# {}\n", comment));
        }

        let prefix = if entry.has_export { "export " } else { "" };

        match entry.quote_style {
            QuoteStyle::Double => {
                let escaped = entry.value.replace('\\', "\\\\").replace('"', "\\\"");
                output.push_str(&format!("{}{}=\"{}\"\n", prefix, entry.key, escaped));
            }
            QuoteStyle::Single => {
                output.push_str(&format!("{}{}='{}'\n", prefix, entry.key, entry.value));
            }
            QuoteStyle::None => {
                let needs_quotes = entry.value.contains(' ')
                    || entry.value.contains('#')
                    || entry.value.contains('"')
                    || entry.value.is_empty();

                if needs_quotes {
                    let escaped = entry.value.replace('\\', "\\\\").replace('"', "\\\"");
                    output.push_str(&format!("{}{}=\"{}\"\n", prefix, entry.key, escaped));
                } else {
                    output.push_str(&format!("{}{}={}\n", prefix, entry.key, entry.value));
                }
            }
        }
    }

    // Atomic write: write to temp file, then rename
    let dir = env_file.path.parent().ok_or("No parent directory")?;
    let temp_path = dir.join(format!(".envi_tmp_{}", std::process::id()));

    fs::write(&temp_path, &output).map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Preserve original file permissions on Unix
    #[cfg(unix)]
    {
        if let Ok(metadata) = fs::metadata(&env_file.path) {
            let _ = fs::set_permissions(&temp_path, metadata.permissions());
        }
    }

    fs::rename(&temp_path, &env_file.path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        format!("Failed to save {}: {}", env_file.path.display(), e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = "KEY=value\nOTHER=123";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries.len(), 2);
        assert_eq!(env.entries[0].key, "KEY");
        assert_eq!(env.entries[0].value, "value");
        assert_eq!(env.entries[1].key, "OTHER");
        assert_eq!(env.entries[1].value, "123");
    }

    #[test]
    fn test_parse_quoted() {
        let content = "KEY=\"hello world\"\nSINGLE='test'";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].value, "hello world");
        assert_eq!(env.entries[1].value, "test");
    }

    #[test]
    fn test_parse_export() {
        let content = "export KEY=value";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].key, "KEY");
        assert_eq!(env.entries[0].value, "value");
    }

    #[test]
    fn test_parse_comments() {
        let content = "# Database config\nDB_HOST=localhost";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].comment, Some("Database config".to_string()));
    }

    #[test]
    fn test_parse_inline_comment() {
        let content = "KEY=value # this is a comment";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].value, "value");
    }

    #[test]
    fn test_parse_encrypted() {
        let content = "SECRET=ENC[age1234abcd]";
        let env = parse_content(content, PathBuf::from(".env"));
        assert!(env.entries[0].is_encrypted);
    }

    #[test]
    fn test_empty_value() {
        let content = "KEY=";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].value, "");
    }

    #[test]
    fn test_parse_value_with_equals_sign() {
        let content = "URL=https://host?a=1&b=2";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].key, "URL");
        assert_eq!(env.entries[0].value, "https://host?a=1&b=2");
    }

    #[test]
    fn test_parse_malformed_no_equals() {
        let content = "NOEQUALS";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries.len(), 0);
        assert_eq!(env.errors.len(), 1);
    }

    #[test]
    fn test_parse_empty_key() {
        let content = "=value";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries.len(), 0);
        assert_eq!(env.errors.len(), 1);
    }

    #[test]
    fn test_parse_empty_content() {
        let content = "";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries.len(), 0);
        assert_eq!(env.errors.len(), 0);
    }

    #[test]
    fn test_parse_preserves_export_flag() {
        let content = "export KEY=value";
        let env = parse_content(content, PathBuf::from(".env"));
        assert!(env.entries[0].has_export);
    }

    #[test]
    fn test_parse_no_export_flag() {
        let content = "KEY=value";
        let env = parse_content(content, PathBuf::from(".env"));
        assert!(!env.entries[0].has_export);
    }

    #[test]
    fn test_parse_double_quote_style() {
        let content = "KEY=\"hello world\"";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].quote_style, QuoteStyle::Double);
        assert_eq!(env.entries[0].value, "hello world");
    }

    #[test]
    fn test_parse_single_quote_style() {
        let content = "KEY='hello world'";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].quote_style, QuoteStyle::Single);
        assert_eq!(env.entries[0].value, "hello world");
    }

    #[test]
    fn test_parse_no_quote_style() {
        let content = "KEY=simple";
        let env = parse_content(content, PathBuf::from(".env"));
        assert_eq!(env.entries[0].quote_style, QuoteStyle::None);
    }

    #[test]
    fn test_write_roundtrip() {
        let dir = std::env::temp_dir().join("envi_test_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");

        let original = "# Comment\nKEY=simple\nSPACED=\"hello world\"\nEMPTY=\n";
        std::fs::write(&path, original).unwrap();

        let env = parse_file(&path).unwrap();
        write_env_file(&env).unwrap();
        let env2 = parse_file(&path).unwrap();

        assert_eq!(env.entries.len(), env2.entries.len());
        for (a, b) in env.entries.iter().zip(env2.entries.iter()) {
            assert_eq!(a.key, b.key);
            assert_eq!(a.value, b.value);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_write_preserves_export() {
        let dir = std::env::temp_dir().join("envi_test_export_rt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");

        let original = "export KEY=value\n";
        std::fs::write(&path, original).unwrap();

        let env = parse_file(&path).unwrap();
        write_env_file(&env).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("export KEY=value"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
