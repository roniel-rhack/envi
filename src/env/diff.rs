use super::parser::EnvFile;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub enum DiffKind {
    Missing,   // Key exists in source but not in target
    Extra,     // Key exists in target but not in source
    Changed,   // Key exists in both but values differ
    Unchanged, // Key exists in both with same value
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub key: String,
    pub kind: DiffKind,
    pub source_value: Option<String>,
    pub target_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub source_name: String,
    pub target_name: String,
    pub entries: Vec<DiffEntry>,
}

impl DiffResult {
    pub fn missing_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.kind == DiffKind::Missing)
            .count()
    }

    pub fn extra_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.kind == DiffKind::Extra)
            .count()
    }

    pub fn changed_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.kind == DiffKind::Changed)
            .count()
    }

    #[allow(dead_code)]
    pub fn has_differences(&self) -> bool {
        self.entries.iter().any(|e| e.kind != DiffKind::Unchanged)
    }
}

pub fn diff_files(source: &EnvFile, target: &EnvFile) -> DiffResult {
    let source_map = source.as_map();
    let target_map = target.as_map();

    let all_keys: BTreeSet<&str> = source_map
        .keys()
        .chain(target_map.keys())
        .copied()
        .collect();

    let mut entries = Vec::new();

    for key in all_keys {
        let source_entry = source_map.get(key);
        let target_entry = target_map.get(key);

        let diff = match (source_entry, target_entry) {
            (Some(s), None) => DiffEntry {
                key: key.to_string(),
                kind: DiffKind::Missing,
                source_value: Some(s.value.clone()),
                target_value: None,
            },
            (None, Some(t)) => DiffEntry {
                key: key.to_string(),
                kind: DiffKind::Extra,
                source_value: None,
                target_value: Some(t.value.clone()),
            },
            (Some(s), Some(t)) => {
                let kind = if s.value == t.value {
                    DiffKind::Unchanged
                } else {
                    DiffKind::Changed
                };
                DiffEntry {
                    key: key.to_string(),
                    kind,
                    source_value: Some(s.value.clone()),
                    target_value: Some(t.value.clone()),
                }
            }
            (None, None) => unreachable!(),
        };

        entries.push(diff);
    }

    // Sort: missing first, then extra, then changed, then unchanged
    entries.sort_by_key(|e| match e.kind {
        DiffKind::Missing => 0,
        DiffKind::Extra => 1,
        DiffKind::Changed => 2,
        DiffKind::Unchanged => 3,
    });

    DiffResult {
        source_name: source.name().to_string(),
        target_name: target.name().to_string(),
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::parser::parse_content;
    use std::path::PathBuf;

    #[test]
    fn test_diff_identical() {
        let a = parse_content("KEY=value\nOTHER=123", PathBuf::from(".env"));
        let b = parse_content("KEY=value\nOTHER=123", PathBuf::from(".env.example"));
        let result = diff_files(&a, &b);
        assert!(!result.has_differences());
    }

    #[test]
    fn test_diff_missing() {
        let a = parse_content("KEY=value\nOTHER=123", PathBuf::from(".env.example"));
        let b = parse_content("KEY=value", PathBuf::from(".env"));
        let result = diff_files(&a, &b);
        assert_eq!(result.missing_count(), 1);
    }

    #[test]
    fn test_diff_extra() {
        let a = parse_content("KEY=value", PathBuf::from(".env.example"));
        let b = parse_content("KEY=value\nEXTRA=new", PathBuf::from(".env"));
        let result = diff_files(&a, &b);
        assert_eq!(result.extra_count(), 1);
    }

    #[test]
    fn test_diff_changed() {
        let a = parse_content("KEY=old", PathBuf::from(".env.example"));
        let b = parse_content("KEY=new", PathBuf::from(".env"));
        let result = diff_files(&a, &b);
        assert_eq!(result.changed_count(), 1);
    }

    #[test]
    fn test_diff_empty_files() {
        let a = parse_content("", PathBuf::from(".env"));
        let b = parse_content("", PathBuf::from(".env.example"));
        let result = diff_files(&a, &b);
        assert!(result.entries.is_empty());
        assert!(!result.has_differences());
    }

    #[test]
    fn test_diff_completely_disjoint() {
        let a = parse_content("A=1\nB=2", PathBuf::from(".env"));
        let b = parse_content("C=3\nD=4", PathBuf::from(".env.example"));
        let result = diff_files(&a, &b);
        assert_eq!(result.missing_count(), 2);
        assert_eq!(result.extra_count(), 2);
        assert_eq!(result.changed_count(), 0);
    }

    #[test]
    fn test_diff_result_names() {
        let a = parse_content("KEY=1", PathBuf::from(".env"));
        let b = parse_content("KEY=1", PathBuf::from(".env.production"));
        let result = diff_files(&a, &b);
        assert_eq!(result.source_name, ".env");
        assert_eq!(result.target_name, ".env.production");
    }

    #[test]
    fn test_diff_sort_order() {
        let a = parse_content("MISSING=1\nCHANGED=old\nSAME=x", PathBuf::from(".env"));
        let b = parse_content(
            "EXTRA=new\nCHANGED=new\nSAME=x",
            PathBuf::from(".env.example"),
        );
        let result = diff_files(&a, &b);

        assert_eq!(result.entries[0].kind, DiffKind::Missing);
        assert_eq!(result.entries[1].kind, DiffKind::Extra);
        assert_eq!(result.entries[2].kind, DiffKind::Changed);
        assert_eq!(result.entries[3].kind, DiffKind::Unchanged);
    }

    #[test]
    fn test_diff_has_differences_positive() {
        let a = parse_content("KEY=old", PathBuf::from(".env"));
        let b = parse_content("KEY=new", PathBuf::from(".env.example"));
        let result = diff_files(&a, &b);
        assert!(result.has_differences());
    }
}
