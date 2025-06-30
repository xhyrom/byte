use crate::compression::{
    CompressionOptions, calculate_compression_ratio, compress_directory, format_file_size,
    get_file_size,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::thread;

pub async fn run(
    source: PathBuf,
    destination: PathBuf,
    quality: f32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Source: {}", source.display());
    println!("Destination: {}", destination.display());
    println!("Quality: {quality}");

    let options = CompressionOptions {
        quality,
        size_factor: 0.8,
    };

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("#>-")
    );

    let pb_clone = pb.clone();
    let progress_callback = Box::new(move |current: usize, total: usize| {
        pb_clone.set_length(total as u64);
        pb_clone.set_position(current as u64);
    });

    let thread_count = thread::available_parallelism()
        .map(|x| x.get())
        .unwrap_or(4)
        .min(8);

    println!("Using {thread_count} threads for compression");

    let compressed_files = compress_directory(
        &source,
        &destination,
        &options,
        thread_count as u32,
        Some(progress_callback),
    )?;

    pb.finish();

    let mut total_original_size = 0;
    let mut total_compressed_size = 0;

    println!();
    println!();
    println!("Files compressed: {}", compressed_files.len());

    for compressed_path in &compressed_files {
        let rel_path = compressed_path
            .strip_prefix(&destination)
            .map_err(|e| e.to_string())?;
        let original_path = source.join(rel_path);

        if original_path.exists() {
            if let (Ok(original_size), Ok(compressed_size)) = (
                get_file_size(&original_path),
                get_file_size(compressed_path),
            ) {
                total_original_size += original_size;
                total_compressed_size += compressed_size;
            }
        }
    }

    println!("Original size: {}", format_file_size(total_original_size));
    println!(
        "Compressed size: {}",
        format_file_size(total_compressed_size)
    );

    let ratio = calculate_compression_ratio(total_original_size, total_compressed_size);
    println!("Compression ratio: {ratio:.1}%");
    println!("Compressed files saved to: {}", destination.display());

    Ok(())
}
