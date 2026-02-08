use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};

use content_inspector::inspect;

use crate::application::selection::collect_selected_files;
use crate::models::{
    ExportConfig, ExportResult, LargeFileStrategy, PreviewMeta, ScanLimits, SelectionSummary,
};

const STREAM_CHUNK_SIZE: usize = 16 * 1024;

pub fn evaluate_selection(config: &ExportConfig, limits: &ScanLimits) -> Result<SelectionSummary, String> {
    let selection = collect_selected_files(config, limits)?;
    Ok(SelectionSummary {
        included_files: selection.included_files,
        excluded_files: selection.excluded_files,
        warnings: selection.warnings,
    })
}

pub fn preview_export(config: &ExportConfig, limits: &ScanLimits) -> Result<PreviewMeta, String> {
    let selection = collect_selected_files(config, limits)?;
    let max_bytes = config.max_file_size_kb.saturating_mul(1024);

    let mut estimated_bytes = 0u64;
    for item in &selection.files {
        match config.large_file_strategy {
            LargeFileStrategy::Truncate => {
                estimated_bytes = estimated_bytes.saturating_add(item.size.min(max_bytes));
            }
            LargeFileStrategy::Skip => {
                if item.size <= max_bytes {
                    estimated_bytes = estimated_bytes.saturating_add(item.size);
                }
            }
        }
    }

    Ok(PreviewMeta {
        included_files: selection.included_files,
        estimated_bytes,
        estimated_tokens: None,
        warnings: selection.warnings,
    })
}

pub fn run_export(
    config: &ExportConfig,
    output_path: &str,
    limits: &ScanLimits,
) -> Result<ExportResult, String> {
    let selection = collect_selected_files(config, limits)?;
    let output_abs = prepare_output_path(output_path)?;

    let parent = output_abs
        .parent()
        .ok_or_else(|| "outputPath must include a parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create output directory: {e}"))?;

    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&output_abs)
        .map_err(|e| format!("Failed to create output file: {e}"))?;

    let mut writer = BufWriter::new(file);
    let mut total_written = 0u64;
    let mut exported_files = 0usize;
    let mut skipped_files = 0usize;
    let mut notes = selection.warnings;

    write_line(&mut writer, "=== STRUCTURE ===", &mut total_written)?;
    for line in build_structure_lines(&selection.files) {
        write_line(&mut writer, &line, &mut total_written)?;
    }
    write_line(&mut writer, "", &mut total_written)?;

    let max_bytes = config.max_file_size_kb.saturating_mul(1024);
    for selected in selection.files {
        let mut file_handle = match File::open(&selected.abs_path) {
            Ok(handle) => handle,
            Err(err) => {
                skipped_files += 1;
                notes.push(format!("Skipped '{}': failed to open ({err})", selected.rel_path));
                continue;
            }
        };

        let mut probe = [0u8; 1024];
        let read_probe = file_handle
            .read(&mut probe)
            .map_err(|e| format!("Failed to inspect file '{}': {e}", selected.rel_path))?;
        if inspect(&probe[..read_probe]).is_binary() {
            skipped_files += 1;
            notes.push(format!("Skipped '{}': binary file", selected.rel_path));
            continue;
        }

        file_handle
            .rewind()
            .map_err(|e| format!("Failed to rewind file '{}': {e}", selected.rel_path))?;

        if matches!(config.large_file_strategy, LargeFileStrategy::Skip) && selected.size > max_bytes {
            skipped_files += 1;
            notes.push(format!(
                "Skipped '{}': exceeds maxFileSizeKB",
                selected.rel_path
            ));
            continue;
        }

        write_line(
            &mut writer,
            &format!("=== FILE: {} ===", selected.rel_path),
            &mut total_written,
        )?;

        if matches!(config.large_file_strategy, LargeFileStrategy::Truncate) && selected.size > max_bytes {
            write_file_content_streaming(
                &mut writer,
                &mut file_handle,
                Some(max_bytes),
                &mut total_written,
            )
            .map_err(|e| format!("Failed to stream file '{}': {e}", selected.rel_path))?;
            write_newline(&mut writer, &mut total_written)?;
            write_line(
                &mut writer,
                &format!("[TRUNCATED at {} bytes]", max_bytes),
                &mut total_written,
            )?;
            notes.push(format!(
                "Truncated '{}': wrote first {} bytes",
                selected.rel_path, max_bytes
            ));
        } else {
            write_file_content_streaming(&mut writer, &mut file_handle, None, &mut total_written)
                .map_err(|e| format!("Failed to stream file '{}': {e}", selected.rel_path))?;
            write_newline(&mut writer, &mut total_written)?;
        }

        write_line(
            &mut writer,
            &format!("=== END FILE: {} ===", selected.rel_path),
            &mut total_written,
        )?;
        write_line(&mut writer, "", &mut total_written)?;
        exported_files += 1;
    }

    writer
        .flush()
        .map_err(|e| format!("Failed to flush output file: {e}"))?;

    Ok(ExportResult {
        output_path: output_abs.to_string_lossy().replace('\\', "/"),
        exported_files,
        skipped_files,
        total_bytes_written: total_written,
        notes,
    })
}

fn prepare_output_path(output_path: &str) -> Result<PathBuf, String> {
    let trimmed = output_path.trim();
    if trimmed.is_empty() {
        return Err("outputPath is required".to_string());
    }
    let candidate = Path::new(trimmed);
    if candidate.file_name().is_none() {
        return Err("outputPath must be a file path, not a directory".to_string());
    }
    if candidate.exists() {
        if candidate.is_dir() {
            return Err("outputPath must be a file path, not a directory".to_string());
        }
        return Err("outputPath already exists; overwrite is disabled by default".to_string());
    }
    Ok(candidate.to_path_buf())
}

fn build_structure_lines(files: &[crate::application::selection::SelectedFile]) -> Vec<String> {
    #[derive(Clone)]
    struct StructureEntry {
        path: String,
        is_dir: bool,
    }

    let mut seen = HashSet::new();
    let mut entries = Vec::new();

    seen.insert(".".to_string());
    entries.push(StructureEntry {
        path: ".".to_string(),
        is_dir: true,
    });

    for file in files {
        let parts = file.rel_path.split('/').collect::<Vec<_>>();
        if parts.is_empty() {
            continue;
        }
        let mut current = String::new();
        for (index, part) in parts.iter().enumerate() {
            if current.is_empty() {
                current.push_str(part);
            } else {
                current.push('/');
                current.push_str(part);
            }
            if seen.insert(current.clone()) {
                entries.push(StructureEntry {
                    path: current.clone(),
                    is_dir: index + 1 != parts.len(),
                });
            }
        }
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => {
            let primary = a.path.to_lowercase().cmp(&b.path.to_lowercase());
            if primary == std::cmp::Ordering::Equal {
                a.path.cmp(&b.path)
            } else {
                primary
            }
        }
    });

    entries.into_iter().map(|entry| entry.path).collect()
}

fn write_file_content_streaming(
    writer: &mut BufWriter<File>,
    file_handle: &mut File,
    max_bytes: Option<u64>,
    total_written: &mut u64,
) -> Result<(), String> {
    let mut raw_buffer = [0u8; STREAM_CHUNK_SIZE];
    let mut normalized_buffer = Vec::with_capacity(STREAM_CHUNK_SIZE + 2);
    let mut utf8_tail: Vec<u8> = Vec::new();
    let mut pending_cr = false;
    let mut remaining = max_bytes;

    loop {
        let to_read = match remaining {
            Some(0) => break,
            Some(bytes_left) => usize::min(bytes_left as usize, raw_buffer.len()),
            None => raw_buffer.len(),
        };

        let read_len = file_handle
            .read(&mut raw_buffer[..to_read])
            .map_err(|e| format!("Failed to read file: {e}"))?;
        if read_len == 0 {
            break;
        }

        if let Some(bytes_left) = &mut remaining {
            *bytes_left = bytes_left.saturating_sub(read_len as u64);
        }

        normalized_buffer.clear();
        normalize_newline_bytes(
            &raw_buffer[..read_len],
            &mut pending_cr,
            &mut normalized_buffer,
        );
        write_utf8_lossy_segment(writer, &normalized_buffer, &mut utf8_tail, total_written)?;
    }

    if pending_cr {
        write_utf8_lossy_segment(writer, b"\n", &mut utf8_tail, total_written)?;
    }

    if !utf8_tail.is_empty() {
        write_utf8_lossy_raw(writer, &utf8_tail, total_written)?;
    }

    Ok(())
}

fn normalize_newline_bytes(input: &[u8], pending_cr: &mut bool, output: &mut Vec<u8>) {
    let mut index = 0usize;

    if *pending_cr {
        if input.first().copied() == Some(b'\n') {
            output.push(b'\n');
            index = 1;
        } else {
            output.push(b'\n');
        }
        *pending_cr = false;
    }

    while index < input.len() {
        let byte = input[index];
        if byte == b'\r' {
            if index + 1 < input.len() {
                if input[index + 1] == b'\n' {
                    index += 1;
                }
                output.push(b'\n');
            } else {
                *pending_cr = true;
            }
        } else {
            output.push(byte);
        }
        index += 1;
    }
}

fn write_utf8_lossy_segment(
    writer: &mut BufWriter<File>,
    segment: &[u8],
    utf8_tail: &mut Vec<u8>,
    total_written: &mut u64,
) -> Result<(), String> {
    if segment.is_empty() {
        return Ok(());
    }

    let mut merged = Vec::with_capacity(utf8_tail.len() + segment.len());
    merged.extend_from_slice(utf8_tail);
    merged.extend_from_slice(segment);

    let split_index = match std::str::from_utf8(&merged) {
        Ok(_) => merged.len(),
        Err(error) => {
            if error.error_len().is_none() {
                error.valid_up_to()
            } else {
                merged.len()
            }
        }
    };

    write_utf8_lossy_raw(writer, &merged[..split_index], total_written)?;
    utf8_tail.clear();
    utf8_tail.extend_from_slice(&merged[split_index..]);
    Ok(())
}

fn write_utf8_lossy_raw(
    writer: &mut BufWriter<File>,
    bytes: &[u8],
    total_written: &mut u64,
) -> Result<(), String> {
    if bytes.is_empty() {
        return Ok(());
    }
    let content = String::from_utf8_lossy(bytes);
    writer
        .write_all(content.as_bytes())
        .map_err(|e| format!("Write failed: {e}"))?;
    *total_written = total_written.saturating_add(content.len() as u64);
    Ok(())
}

fn write_newline(writer: &mut BufWriter<File>, total_written: &mut u64) -> Result<(), String> {
    writer
        .write_all(b"\n")
        .map_err(|e| format!("Write failed: {e}"))?;
    *total_written = total_written.saturating_add(1);
    Ok(())
}

fn write_line(writer: &mut BufWriter<File>, line: &str, total_written: &mut u64) -> Result<(), String> {
    writer
        .write_all(line.as_bytes())
        .map_err(|e| format!("Write failed: {e}"))?;
    *total_written = total_written.saturating_add(line.len() as u64);
    write_newline(writer, total_written)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use tempfile::tempdir;

    use crate::models::{ExportConfig, LargeFileStrategy, OutputFormat, ScanLimits};

    use super::run_export;

    fn test_config(root_path: &str, strategy: LargeFileStrategy, max_file_size_kb: u64) -> ExportConfig {
        ExportConfig {
            root_path: root_path.to_string(),
            use_gitignore: false,
            include_globs: vec![],
            exclude_globs: vec![],
            include_extensions: vec![],
            exclude_extensions: vec![],
            max_file_size_kb,
            large_file_strategy: strategy,
            manual_selections: BTreeMap::new(),
            output_format: OutputFormat::Txt,
        }
    }

    #[test]
    fn rejects_existing_output_file_by_default() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("input.txt"), "hello").unwrap();

        let output_dir = tempdir().unwrap();
        let output_path = output_dir.path().join("existing.txt");
        fs::write(&output_path, "existing").unwrap();

        let config = test_config(root.path().to_string_lossy().as_ref(), LargeFileStrategy::Truncate, 256);
        let result = run_export(
            &config,
            output_path.to_string_lossy().as_ref(),
            &ScanLimits::default(),
        );

        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .contains("overwrite is disabled by default"));
    }

    #[test]
    fn rejects_directory_output_path() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("input.txt"), "hello").unwrap();

        let output_dir = tempdir().unwrap();
        let config = test_config(root.path().to_string_lossy().as_ref(), LargeFileStrategy::Truncate, 256);
        let result = run_export(
            &config,
            output_dir.path().to_string_lossy().as_ref(),
            &ScanLimits::default(),
        );

        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .contains("outputPath must be a file path, not a directory"));
    }

    #[test]
    fn truncate_strategy_writes_marker_and_note_for_large_file() {
        let root = tempdir().unwrap();
        let large = "x".repeat(2048);
        fs::write(root.path().join("large.txt"), large).unwrap();

        let output_dir = tempdir().unwrap();
        let output_path = output_dir.path().join("truncate.txt");
        let config = test_config(root.path().to_string_lossy().as_ref(), LargeFileStrategy::Truncate, 1);
        let result = run_export(
            &config,
            output_path.to_string_lossy().as_ref(),
            &ScanLimits::default(),
        )
        .unwrap();

        let output = fs::read_to_string(output_path).unwrap();
        assert_eq!(result.exported_files, 1);
        assert!(output.contains("[TRUNCATED at 1024 bytes]"));
        assert!(result
            .notes
            .iter()
            .any(|note| note.contains("Truncated 'large.txt'")));
    }

    #[test]
    fn skip_strategy_skips_large_file_and_records_note() {
        let root = tempdir().unwrap();
        let large = "x".repeat(2048);
        fs::write(root.path().join("large.txt"), large).unwrap();

        let output_dir = tempdir().unwrap();
        let output_path = output_dir.path().join("skip.txt");
        let config = test_config(root.path().to_string_lossy().as_ref(), LargeFileStrategy::Skip, 1);
        let result = run_export(
            &config,
            output_path.to_string_lossy().as_ref(),
            &ScanLimits::default(),
        )
        .unwrap();

        assert_eq!(result.exported_files, 0);
        assert_eq!(result.skipped_files, 1);
        assert!(result
            .notes
            .iter()
            .any(|note| note.contains("exceeds maxFileSizeKB")));
    }

    #[test]
    fn exported_content_uses_lf_newlines_only() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("mixed.txt"), "a\r\nb\rc\n").unwrap();

        let output_dir = tempdir().unwrap();
        let output_path = output_dir.path().join("newlines.txt");
        let config = test_config(root.path().to_string_lossy().as_ref(), LargeFileStrategy::Truncate, 256);
        run_export(
            &config,
            output_path.to_string_lossy().as_ref(),
            &ScanLimits::default(),
        )
        .unwrap();

        let output = fs::read(output_path).unwrap();
        assert!(!output.iter().any(|byte| *byte == b'\r'));
    }

    #[test]
    fn repeated_exports_have_stable_ordering() {
        let root = tempdir().unwrap();
        fs::create_dir_all(root.path().join("bDir")).unwrap();
        fs::create_dir_all(root.path().join("ADir")).unwrap();
        fs::write(root.path().join("bDir").join("z.txt"), "z").unwrap();
        fs::write(root.path().join("ADir").join("a.txt"), "a").unwrap();
        fs::write(root.path().join("Beta.txt"), "beta").unwrap();
        fs::write(root.path().join("alpha.txt"), "alpha").unwrap();

        let output_dir = tempdir().unwrap();
        let first = output_dir.path().join("first.txt");
        let second = output_dir.path().join("second.txt");
        let config = test_config(root.path().to_string_lossy().as_ref(), LargeFileStrategy::Truncate, 256);

        run_export(
            &config,
            first.to_string_lossy().as_ref(),
            &ScanLimits::default(),
        )
        .unwrap();
        run_export(
            &config,
            second.to_string_lossy().as_ref(),
            &ScanLimits::default(),
        )
        .unwrap();

        let first_output = fs::read(first).unwrap();
        let second_output = fs::read(second).unwrap();
        assert_eq!(first_output, second_output);
    }
}
