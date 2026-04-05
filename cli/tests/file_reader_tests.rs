// Tests for file_reader module - File format handling

mod common;

use std::path::Path;
use std::io::Write;

use seed_intelligence::file_reader::{
    is_ebook_format,
    is_pdf_format,
    check_calibre,
    check_pdftotext,
    read_file,
    stream_read_file,
    cleanup_temp_file,
};

/// Create a temporary file for testing
fn create_temp_file(content: &str, extension: &str) -> std::path::PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("seed_test_{}.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        extension
    );
    let path = temp_dir.join(file_name);

    let mut file = std::fs::File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes()).expect("Failed to write to temp file");

    path
}

#[test]
fn test_is_ebook_format_epub() {
    assert!(is_ebook_format(Path::new("book.epub")));
}

#[test]
fn test_is_ebook_format_mobi() {
    assert!(is_ebook_format(Path::new("book.mobi")));
}

#[test]
fn test_is_ebook_format_azw3() {
    assert!(is_ebook_format(Path::new("book.azw3")));
}

#[test]
fn test_is_ebook_format_azw() {
    assert!(is_ebook_format(Path::new("book.azw")));
}

#[test]
fn test_is_ebook_format_txt() {
    assert!(!is_ebook_format(Path::new("book.txt")));
}

#[test]
fn test_is_ebook_format_no_extension() {
    assert!(!is_ebook_format(Path::new("book")));
}

#[test]
fn test_is_ebook_format_case_insensitive() {
    assert!(is_ebook_format(Path::new("book.EPUB")));
    assert!(is_ebook_format(Path::new("book.Mobi")));
}

#[test]
fn test_is_pdf_format() {
    assert!(is_pdf_format(Path::new("document.pdf")));
}

#[test]
fn test_is_pdf_format_not_pdf() {
    assert!(!is_pdf_format(Path::new("document.txt")));
}

#[test]
fn test_is_pdf_format_case_insensitive() {
    assert!(is_pdf_format(Path::new("document.PDF")));
}

#[test]
fn test_check_calibre() {
    // This test just checks if calibre is installed
    // It should not panic either way
    let result = check_calibre();
    println!("Calibre installed: {}", result);
}

#[test]
fn test_check_pdftotext() {
    // This test just checks if pdftotext is installed
    let result = check_pdftotext();
    println!("pdftotext installed: {}", result);
}

#[test]
fn test_read_file_txt() {
    let content = "这是一个测试文件\n包含多行内容";
    let path = create_temp_file(content, "txt");

    let result = read_file(&path);

    assert!(result.is_ok());
    let (text, temp_file) = result.unwrap();
    assert!(text.contains("测试文件"));
    assert!(temp_file.is_none()); // No conversion needed for txt

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn test_read_file_not_found() {
    let path = std::path::PathBuf::from("/nonexistent/path/file.txt");

    let result = read_file(&path);

    assert!(result.is_err());
}

#[test]
fn test_stream_read_file_txt() {
    let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
    let path = create_temp_file(content, "txt");

    let mut lines_received = Vec::new();

    let result = stream_read_file(&path, 2, |text| {
        lines_received.push(text.to_string());
    });

    assert!(result.is_ok());
    let (lines_processed, temp_file) = result.unwrap();
    assert!(lines_processed > 0);
    assert!(temp_file.is_none()); // No conversion for txt

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn test_stream_read_file_with_filter() {
    let content = "Short\n\nThis is a longer line that should be processed\nAnother good line here";
    let path = create_temp_file(content, "txt");

    let mut callbacks = 0;

    let result = stream_read_file(&path, 1, |_text| {
        callbacks += 1;
    });

    assert!(result.is_ok());

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn test_stream_read_file_not_found() {
    let path = std::path::PathBuf::from("/nonexistent/stream.txt");

    let result = stream_read_file(&path, 10, |_text| {});

    assert!(result.is_err());
}

#[test]
fn test_cleanup_temp_file_existing() {
    let content = "test";
    let path = create_temp_file(content, "tmp");

    assert!(path.exists());

    cleanup_temp_file(Some(&path));

    assert!(!path.exists());
}

#[test]
fn test_cleanup_temp_file_nonexistent() {
    let path = std::path::PathBuf::from("/nonexistent/temp.txt");

    // Should not panic
    cleanup_temp_file(Some(&path));
}

#[test]
fn test_cleanup_temp_file_none() {
    // Should not panic
    cleanup_temp_file(None);
}

#[test]
fn test_read_file_chinese_content() {
    let content = "人工智能是计算机科学的一个分支\n机器学习是人工智能的核心技术";
    let path = create_temp_file(content, "txt");

    let result = read_file(&path);

    assert!(result.is_ok());
    let (text, _) = result.unwrap();
    assert!(text.contains("人工智能"));
    assert!(text.contains("机器学习"));

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn test_read_file_empty() {
    let content = "";
    let path = create_temp_file(content, "txt");

    let result = read_file(&path);

    assert!(result.is_ok());

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn test_read_file_large_content() {
    // Create a larger file to test streaming
    let content: String = (0..100).map(|i| format!("这是第{}行内容\n", i)).collect();
    let path = create_temp_file(&content, "txt");

    let result = read_file(&path);

    assert!(result.is_ok());
    let (text, _) = result.unwrap();
    assert!(text.contains("第0行"));
    assert!(text.contains("第99行"));

    // Cleanup
    std::fs::remove_file(path).ok();
}

// Note: EPUB/PDF conversion tests require external tools
// These are integration tests that should be run with --ignored flag

#[test]
#[ignore]
fn test_read_file_epub() {
    // This requires calibre to be installed
    // Place a test.epub file in temp directory
    let path = std::env::temp_dir().join("test.epub");

    if path.exists() {
        let result = read_file(&path);
        // May fail if calibre is not installed
        println!("EPUB read result: {:?}", result.is_ok());
    }
}

#[test]
#[ignore]
fn test_read_file_pdf() {
    // This requires poppler-utils (pdftotext) to be installed
    let path = std::env::temp_dir().join("test.pdf");

    if path.exists() {
        let result = read_file(&path);
        println!("PDF read result: {:?}", result.is_ok());
    }
}
