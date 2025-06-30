use anyhow::Result;
use image_compressor::{Factor, FolderCompressor};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub struct CompressionOptions {
    pub quality: f32,
    pub size_factor: f32,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            quality: 80.,
            size_factor: 0.8,
        }
    }
}

pub fn compress_directory(
    source_dir: &Path,
    dest_dir: &Path,
    options: &CompressionOptions,
    thread_count: u32,
    progress_callback: Option<Box<dyn Fn(usize, usize) + Send>>,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(dest_dir)?;

    let (tx, tr) = mpsc::channel();
    let mut comp = FolderCompressor::new(source_dir, dest_dir);
    comp.set_factor(Factor::new(options.quality, options.size_factor));
    comp.set_thread_count(thread_count);
    comp.set_sender(tx);

    let handle = std::thread::spawn(move || comp.compress().map_err(|e| anyhow::anyhow!("{}", e)));

    let mut total_files = 0;
    let mut compressed_files = Vec::new();
    for (i, msg) in tr.iter().enumerate() {
        if msg.starts_with("Total file count: ") {
            total_files = msg.replace("Total file count: ", "").parse().unwrap_or(0);

            if let Some(callback) = &progress_callback {
                callback(0, total_files);
            }

            continue;
        }

        if msg.starts_with("Compress complete! File:")
            || msg.starts_with("A file with the same name exists: ")
        {
            let file_name = if msg.starts_with("Compress complete") {
                msg.replace("Compress complete! File: ", "")
            } else {
                msg.replace("A file with the same name exists: ", "")
            };
            let dest_path = dest_dir.join(&file_name);

            compressed_files.push(dest_path);
            if let Some(callback) = &progress_callback {
                callback(i, total_files);
            }
        }
    }

    match handle.join().unwrap() {
        Ok(_) => Ok(compressed_files),
        Err(e) => Err(anyhow::anyhow!("Compression failed: {}", e)),
    }
}

pub fn get_file_size(path: &Path) -> Result<u64, Box<dyn Error + Send + Sync>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{size} bytes")
    }
}

pub fn calculate_compression_ratio(original: u64, compressed: u64) -> f64 {
    if original == 0 {
        return 0.0;
    }
    let reduction = original as f64 - compressed as f64;
    (reduction / original as f64) * 100.0
}
