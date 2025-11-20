//! JSONL (JSON Lines) file handling utilities

use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::error::{ColonyError, ColonyResult};

/// Read all entries from a JSONL file
pub async fn read_jsonl<T: DeserializeOwned>(path: &Path) -> ColonyResult<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)
        .await
        .map_err(|e| ColonyError::Colony(format!("Failed to open JSONL file {:?}: {}", path, e)))?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut entries = Vec::new();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| ColonyError::Colony(format!("Failed to read line from {:?}: {}", path, e)))?
    {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry: T = serde_json::from_str(line).map_err(|e| {
            ColonyError::Colony(format!("Failed to parse JSON from {:?}: {}", path, e))
        })?;

        entries.push(entry);
    }

    Ok(entries)
}

/// Write entries to a JSONL file
pub async fn write_jsonl<T: Serialize>(path: &Path, entries: &[T]) -> ColonyResult<()> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            ColonyError::Colony(format!("Failed to create directory {:?}: {}", parent, e))
        })?;
    }

    let mut file = File::create(path).await.map_err(|e| {
        ColonyError::Colony(format!("Failed to create JSONL file {:?}: {}", path, e))
    })?;

    for entry in entries {
        let json = serde_json::to_string(entry)
            .map_err(|e| ColonyError::Colony(format!("Failed to serialize entry: {}", e)))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| ColonyError::Colony(format!("Failed to write to {:?}: {}", path, e)))?;

        file.write_all(b"\n").await.map_err(|e| {
            ColonyError::Colony(format!("Failed to write newline to {:?}: {}", path, e))
        })?;
    }

    file.sync_all()
        .await
        .map_err(|e| ColonyError::Colony(format!("Failed to sync {:?}: {}", path, e)))?;

    Ok(())
}

/// Append a single entry to a JSONL file
pub async fn append_jsonl<T: Serialize>(path: &Path, entry: &T) -> ColonyResult<()> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            ColonyError::Colony(format!("Failed to create directory {:?}: {}", parent, e))
        })?;
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|e| ColonyError::Colony(format!("Failed to open {:?}: {}", path, e)))?;

    let json = serde_json::to_string(entry)
        .map_err(|e| ColonyError::Colony(format!("Failed to serialize entry: {}", e)))?;

    file.write_all(json.as_bytes())
        .await
        .map_err(|e| ColonyError::Colony(format!("Failed to write to {:?}: {}", path, e)))?;

    file.write_all(b"\n").await.map_err(|e| {
        ColonyError::Colony(format!("Failed to write newline to {:?}: {}", path, e))
    })?;

    file.sync_all()
        .await
        .map_err(|e| ColonyError::Colony(format!("Failed to sync {:?}: {}", path, e)))?;

    Ok(())
}

/// Get the last modified time of a file
pub async fn get_modified_time(path: &Path) -> ColonyResult<std::time::SystemTime> {
    let metadata = tokio::fs::metadata(path).await.map_err(|e| {
        ColonyError::Colony(format!("Failed to get metadata for {:?}: {}", path, e))
    })?;

    metadata.modified().map_err(|e| {
        ColonyError::Colony(format!("Failed to get modified time for {:?}: {}", path, e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEntry {
        id: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_write_and_read_jsonl() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.jsonl");

        let entries = vec![
            TestEntry {
                id: "1".to_string(),
                value: 100,
            },
            TestEntry {
                id: "2".to_string(),
                value: 200,
            },
        ];

        write_jsonl(&file_path, &entries).await.unwrap();

        let read_entries: Vec<TestEntry> = read_jsonl(&file_path).await.unwrap();

        assert_eq!(entries, read_entries);
    }

    #[tokio::test]
    async fn test_append_jsonl() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.jsonl");

        let entry1 = TestEntry {
            id: "1".to_string(),
            value: 100,
        };
        let entry2 = TestEntry {
            id: "2".to_string(),
            value: 200,
        };

        append_jsonl(&file_path, &entry1).await.unwrap();
        append_jsonl(&file_path, &entry2).await.unwrap();

        let entries: Vec<TestEntry> = read_jsonl(&file_path).await.unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry1);
        assert_eq!(entries[1], entry2);
    }

    #[tokio::test]
    async fn test_read_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.jsonl");

        let entries: Vec<TestEntry> = read_jsonl(&file_path).await.unwrap();

        assert!(entries.is_empty());
    }
}
