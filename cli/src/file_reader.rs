// File Reader Module - Supports txt, epub, mobi, azw3, pdf formats
// Uses Calibre for ebook conversion, pdftotext for PDF

use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{BufRead, BufReader, Read, Write};
use std::fs::File;
use sha2::{Sha256, Digest};

use crate::brain::BookMetadata;

/// Supported ebook formats
const EBOOK_FORMATS: [&str; 4] = ["epub", "mobi", "azw3", "azw"];

/// Check if Calibre is available
pub fn check_calibre() -> bool {
    Command::new("ebook-convert")
        .arg("--version")
        .output()
        .is_ok()
}

/// Check if pdftotext is available (for PDF)
pub fn check_pdftotext() -> bool {
    Command::new("pdftotext")
        .arg("-v")
        .output()
        .is_ok()
}

/// Check if file is an ebook format
pub fn is_ebook_format(path: &Path) -> bool {
    path.extension()
        .map(|ext| {
            let ext = ext.to_string_lossy().to_lowercase();
            EBOOK_FORMATS.contains(&ext.as_str())
        })
        .unwrap_or(false)
}

/// Check if file is a PDF
pub fn is_pdf_format(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_lowercase() == "pdf")
        .unwrap_or(false)
}

/// Check if PDF is text-based (not image-only)
pub fn is_text_pdf(path: &Path) -> bool {
    if !check_pdftotext() {
        return false;
    }

    let output = Command::new("pdftotext")
        .arg(path)
        .arg("-") // Output to stdout
        .output();

    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            // If we got more than 100 chars of meaningful text, it's text-based
            text.chars().filter(|c| !c.is_whitespace()).count() > 100
        }
        Err(_) => false,
    }
}

/// Convert ebook to text using Calibre
pub fn convert_ebook_to_txt(input_path: &Path) -> Result<PathBuf, String> {
    if !check_calibre() {
        return Err("Calibre not installed. Run: sudo apt install calibre".to_string());
    }

    let temp_dir = std::env::temp_dir();
    let base_name = input_path
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "ebook".to_string());

    let output_path = temp_dir.join(format!("{}_{}.txt", base_name, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)));

    println!("[FileReader] Converting ebook: {:?} -> {:?}", input_path, output_path);

    let output = Command::new("ebook-convert")
        .arg(input_path)
        .arg(&output_path)
        .output()
        .map_err(|e| format!("Failed to run ebook-convert: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Conversion failed: {}", stderr));
    }

    if !output_path.exists() {
        return Err("Output file not created".to_string());
    }

    Ok(output_path)
}

/// Convert PDF to text using pdftotext
pub fn convert_pdf_to_txt(input_path: &Path) -> Result<PathBuf, String> {
    if !check_pdftotext() {
        return Err("pdftotext not installed. Run: sudo apt install poppler-utils".to_string());
    }

    // First check if it's a text-based PDF
    if !is_text_pdf(input_path) {
        return Err("PDF appears to be image-based (scanned). OCR would be required.".to_string());
    }

    let temp_dir = std::env::temp_dir();
    let base_name = input_path
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "pdf".to_string());

    let output_path = temp_dir.join(format!("{}_{}.txt", base_name, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)));

    println!("[FileReader] Converting PDF: {:?} -> {:?}", input_path, output_path);

    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg(input_path)
        .arg(&output_path)
        .output()
        .map_err(|e| format!("Failed to run pdftotext: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PDF conversion failed: {}", stderr));
    }

    if !output_path.exists() {
        return Err("Output file not created".to_string());
    }

    Ok(output_path)
}

/// Convert any supported format to text
/// Returns path to temp txt file if conversion was needed
pub fn convert_to_txt(input_path: &Path) -> Result<PathBuf, String> {
    if is_ebook_format(input_path) {
        convert_ebook_to_txt(input_path)
    } else if is_pdf_format(input_path) {
        convert_pdf_to_txt(input_path)
    } else {
        Err(format!("Unsupported format: {:?}", input_path.extension()))
    }
}

/// Read file content as text
/// For ebooks/PDFs, converts to temp txt file first
pub fn read_file(path: &Path) -> Result<(String, Option<PathBuf>), String> {
    if !path.exists() {
        return Err(format!("File not found: {:?}", path));
    }

    if is_ebook_format(path) || is_pdf_format(path) {
        let txt_path = convert_to_txt(path)?;
        let content = std::fs::read_to_string(&txt_path)
            .map_err(|e| format!("Failed to read converted file: {}", e))?;
        Ok((content, Some(txt_path)))
    } else {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        Ok((content, None))
    }
}

/// Stream read file with callback (for large files)
/// Returns (lines_processed, temp_file_path if any)
pub fn stream_read_file<F>(
    path: &Path,
    batch_size: usize,
    mut callback: F,
) -> Result<(usize, Option<PathBuf>), String>
where
    F: FnMut(&str),
{
    if !path.exists() {
        return Err(format!("File not found: {:?}", path));
    }

    let (actual_path, temp_file) = if is_ebook_format(path) || is_pdf_format(path) {
        let txt_path = convert_to_txt(path)?;
        println!("[FileReader] Conversion complete, streaming content...");
        (txt_path.clone(), Some(txt_path))
    } else {
        (path.to_path_buf(), None)
    };

    let file = File::open(&actual_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let reader = BufReader::new(file);
    let mut batch = Vec::new();
    let mut lines_processed = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();
        if trimmed.len() < 5 {
            continue;
        }

        batch.push(trimmed.to_string());

        if batch.len() >= batch_size {
            let text = batch.join(" ");
            callback(&text);
            lines_processed += batch.len();
            batch.clear();

            if lines_processed % 1000 < batch_size {
                print!("\r[FileReader] Processed: {} lines...", lines_processed);
                std::io::stdout().flush().ok();
            }
        }
    }

    if !batch.is_empty() {
        let text = batch.join(" ");
        callback(&text);
        lines_processed += batch.len();
    }

    println!();

    Ok((lines_processed, temp_file))
}

/// Clean up temporary file
pub fn cleanup_temp_file(path: Option<&PathBuf>) {
    if let Some(p) = path {
        if p.exists() {
            if let Err(e) = std::fs::remove_file(p) {
                eprintln!("[FileReader] Failed to cleanup temp file: {}", e);
            } else {
                println!("[FileReader] Cleaned up temp file: {:?}", p);
            }
        }
    }
}

// =============================================================================
// Hash Computation
// =============================================================================

/// Compute SHA-256 hash of a file
pub fn compute_file_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file for hashing: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("Failed to read file for hashing: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute SHA-256 hash of text content
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get file size in bytes
pub fn get_file_size(path: &Path) -> Result<i64, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    Ok(metadata.len() as i64)
}

// =============================================================================
// Metadata Extraction
// =============================================================================

/// Extract book metadata from file
pub fn extract_book_metadata(path: &Path) -> BookMetadata {
    let format = get_file_format(path);
    let filename = path
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // Try to extract metadata based on format
    match format.as_str() {
        "epub" => extract_epub_metadata(path).unwrap_or_else(|| BookMetadata {
            title: filename,
            author: None,
            format,
        }),
        "pdf" => extract_pdf_metadata(path).unwrap_or_else(|| BookMetadata {
            title: filename,
            author: None,
            format,
        }),
        _ => BookMetadata {
            title: filename,
            author: None,
            format,
        },
    }
}

/// Get file format from extension
pub fn get_file_format(path: &Path) -> String {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Extract metadata from EPUB file
fn extract_epub_metadata(path: &Path) -> Option<BookMetadata> {
    // EPUB is a ZIP file, we need to find and parse the OPF file
    // For simplicity, we'll use the epub crate if available
    // Otherwise fallback to filename

    // Try using epub crate
    match epub::doc::EpubDoc::new(path) {
        Ok(mut doc) => {
            // mdata returns Option<MetadataItem>, need to convert to String
            let title = doc.mdata("title")
                .map(|item| format!("{:?}", item))
                .unwrap_or_else(|| {
                    path.file_stem()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                });

            let author = doc.mdata("creator")
                .map(|item| format!("{:?}", item));

            Some(BookMetadata {
                title,
                author,
                format: "epub".to_string(),
            })
        }
        Err(_) => None,
    }
}

/// Extract metadata from PDF file using pdftotext
fn extract_pdf_metadata(path: &Path) -> Option<BookMetadata> {
    // Use pdfinfo if available to extract metadata
    let output = Command::new("pdfinfo")
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let info = String::from_utf8_lossy(&output.stdout);
    let mut title: Option<String> = None;
    let mut author: Option<String> = None;

    for line in info.lines() {
        if line.starts_with("Title:") {
            title = Some(line["Title:".len()..].trim().to_string());
        } else if line.starts_with("Author:") {
            author = Some(line["Author:".len()..].trim().to_string());
        }
    }

    let filename = path
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Some(BookMetadata {
        title: title.unwrap_or(filename),
        author,
        format: "pdf".to_string(),
    })
}

// =============================================================================
// Combined Hash and Content Processing
// =============================================================================

/// Read file and compute hash in one pass
/// Returns (content, hash, temp_file_path if any)
pub fn read_file_with_hash(path: &Path) -> Result<(String, String, Option<PathBuf>), String> {
    let (content, temp_file) = read_file(path)?;

    // Hash the content (for ebooks/PDFs, hash the converted text)
    let hash = compute_content_hash(&content);

    Ok((content, hash, temp_file))
}

/// Stream read file with hash computation
/// Returns (lines_processed, content_hash, temp_file_path if any)
pub fn stream_read_file_with_hash<F>(
    path: &Path,
    batch_size: usize,
    mut callback: F,
) -> Result<(usize, String, Option<PathBuf>), String>
where
    F: FnMut(&str),
{
    if !path.exists() {
        return Err(format!("File not found: {:?}", path));
    }

    let (actual_path, temp_file) = if is_ebook_format(path) || is_pdf_format(path) {
        let txt_path = convert_to_txt(path)?;
        println!("[FileReader] Conversion complete, streaming content...");
        (txt_path.clone(), Some(txt_path))
    } else {
        (path.to_path_buf(), None)
    };

    let file = File::open(&actual_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let reader = BufReader::new(file);
    let mut batch = Vec::new();
    let mut lines_processed = 0;
    let mut hasher = Sha256::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();
        if trimmed.len() < 5 {
            continue;
        }

        // Update hash
        hasher.update(trimmed.as_bytes());
        hasher.update(b"\n");

        batch.push(trimmed.to_string());

        if batch.len() >= batch_size {
            let text = batch.join(" ");
            callback(&text);
            lines_processed += batch.len();
            batch.clear();

            if lines_processed % 1000 < batch_size {
                print!("\r[FileReader] Processed: {} lines...", lines_processed);
                std::io::stdout().flush().ok();
            }
        }
    }

    if !batch.is_empty() {
        let text = batch.join(" ");
        callback(&text);
        lines_processed += batch.len();
    }

    println!();

    let hash = format!("{:x}", hasher.finalize());

    Ok((lines_processed, hash, temp_file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ebook_format() {
        assert!(is_ebook_format(Path::new("book.epub")));
        assert!(is_ebook_format(Path::new("book.mobi")));
        assert!(is_ebook_format(Path::new("book.azw3")));
        assert!(!is_ebook_format(Path::new("book.txt")));
    }

    #[test]
    fn test_is_pdf_format() {
        assert!(is_pdf_format(Path::new("document.pdf")));
        assert!(!is_pdf_format(Path::new("document.txt")));
    }

    #[test]
    fn test_compute_content_hash() {
        let hash1 = compute_content_hash("test content");
        let hash2 = compute_content_hash("test content");
        let hash3 = compute_content_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_get_file_format() {
        assert_eq!(get_file_format(Path::new("book.epub")), "epub");
        assert_eq!(get_file_format(Path::new("book.PDF")), "pdf");
        assert_eq!(get_file_format(Path::new("book.txt")), "txt");
    }
}
