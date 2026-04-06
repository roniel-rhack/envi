use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
    pub line_number: usize,
    pub is_encrypted: bool,
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
        let line_content = trimmed.strip_prefix("export ").unwrap_or(trimmed);

        match parse_line(line_content, line_number) {
            Ok(mut entry) => {
                entry.comment = pending_comment.take();
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
    let (value, is_encrypted) = parse_value(raw_value);

    Ok(EnvEntry {
        key,
        value,
        comment: None,
        line_number,
        is_encrypted,
    })
}

fn parse_value(raw: &str) -> (String, bool) {
    let is_encrypted = raw.starts_with("ENC[") && raw.ends_with(']');

    let value = if (raw.starts_with('"') && raw.ends_with('"'))
        || (raw.starts_with('\'') && raw.ends_with('\''))
    {
        if raw.len() >= 2 {
            raw[1..raw.len() - 1].to_string()
        } else {
            raw.to_string()
        }
    } else {
        // Strip inline comments (but not inside quotes)
        raw.split(" #").next().unwrap_or(raw).trim().to_string()
    };

    (value, is_encrypted)
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
        let needs_quotes = entry.value.contains(' ')
            || entry.value.contains('#')
            || entry.value.contains('"')
            || entry.value.is_empty();

        if needs_quotes {
            let escaped = entry.value.replace('"', "\\\"");
            output.push_str(&format!("{}=\"{}\"\n", entry.key, escaped));
        } else {
            output.push_str(&format!("{}={}\n", entry.key, entry.value));
        }
    }

    fs::write(&env_file.path, output)
        .map_err(|e| format!("Failed to write {}: {}", env_file.path.display(), e))
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
}
