# epubie-lib

A Rust library for parsing and manipulating EPUB files. This library provides a simple and efficient way to extract metadata, chapters, table of contents, and file contents from EPUB documents.

## Features

- ✅ Parse EPUB metadata (title, author, language, etc.)
- ✅ Extract chapters and their content
- ✅ Generate table of contents
- ✅ Access individual files within the EPUB
- ✅ HTML content parsing and extraction
- ✅ Support for and 3.0 format

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
epubie-lib = "0.1.0"
```

## Quick Start

```rust
use epubie_lib::Epub;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an EPUB file
    let epub = Epub::new("path/to/your/book.epub".to_string())?;
    
    // Get basic metadata
    println!("Title: {}", epub.get_title());
    println!("Author: {}", epub.get_creator());
    
    // Iterate through chapters
    for (i, chapter) in epub.get_chapters().iter().enumerate() {
        println!("Chapter {}: {}", i + 1, chapter.get_title());
        
        // Access files in each chapter
        for file in chapter.get_files() {
            if file.is_html() {
                println!("  HTML content: {} bytes", file.get_html_bytes().len());
            }
        }
    }
    
    Ok(())
}
```

## Examples

### Extracting Metadata

```rust
use epubie_lib::Epub;

let epub = Epub::new("book.epub".to_string())?;

println!("Title: {}", epub.get_title());
println!("Creator: {}", epub.get_creator());
println!("Language: {}", epub.get_language());
println!("Identifier: {}", epub.get_identifier());
println!("Publication Date: {}", epub.get_date());

if let Some(description) = epub.get_description() {
    println!("Description: {}", description);
}
```

### Working with Chapters

```rust
use epubie_lib::Epub;

let epub = Epub::new("book.epub".to_string())?;

println!("Total chapters: {}", epub.get_chapter_count());

for (i, chapter) in epub.get_chapters().iter().enumerate() {
    println!("Chapter {}: {}", i + 1, chapter.get_title());
    println!("  Files: {}", chapter.get_file_count());
    
    for file in chapter.get_files() {
        println!("    - {} ({})", 
                 file.get_title().unwrap_or("Untitled"), 
                 file.get_href());
    }
}
```

### Table of Contents

```rust
use epubie_lib::Epub;

let epub = Epub::new("book.epub".to_string())?;
let toc = epub.get_table_of_contents();

println!("Table of Contents ({} entries):", toc.get_entry_count());

for entry in toc.get_entries() {
    let indent = "  ".repeat(entry.get_level() as usize);
    println!("{}{} -> {}", indent, entry.get_title(), entry.get_href());
}
```

### Accessing File Contents

```rust
use epubie_lib::Epub;

let epub = Epub::new("book.epub".to_string())?;

for file in epub.get_all_files() {
    println!("File: {} ({})", file.get_href(), file.get_media_type());
    
    if file.is_html() {
        // Get raw HTML content
        let html_content = file.get_html_bytes();
        println!("  HTML size: {} bytes", html_content.len());
        
        // Get parsable HTML (if needed for further processing)
        if let Some(parsed_html) = file.get_parsable_html() {
            println!("  Parsed HTML available");
        }
    }
}
```

## API Reference

### `Epub`

The main struct for working with EPUB files.

#### Methods

- `new(file_path: String) -> Result<Epub, Box<dyn std::error::Error>>` - Create a new EPUB instance
- `get_title() -> &str` - Get the book title
- `get_creator() -> &str` - Get the book author/creator
- `get_language() -> &str` - Get the book language
- `get_identifier() -> &str` - Get the book identifier
- `get_date() -> &str` - Get the publication date
- `get_publisher() -> Option<String>` - Get the publisher
- `get_description() -> Option<String>` - Get the book description
- `get_rights() -> Option<String>` - Get the rights information
- `get_cover() -> Option<String>` - Get the cover image path
- `get_tags() -> Option<Vec<String>>` - Get book tags
- `get_chapters() -> &Vec<Chapter>` - Get all chapters
- `get_chapter_count() -> usize` - Get the number of chapters
- `get_table_of_contents() -> &TableOfContents` - Get the table of contents
- `get_all_files() -> &Vec<EpubFile>` - Get all files in the EPUB
- `get_file_count() -> usize` - Get the total number of files

### `Chapter`

Represents a chapter in the EPUB.

#### Methods

- `get_title() -> &str` - Get the chapter title
- `get_files() -> &Vec<EpubFile>` - Get files in this chapter
- `get_file_count() -> usize` - Get the number of files in this chapter

### `EpubFile`

Represents a file within the EPUB.

#### Methods

- `get_id() -> &str` - Get the file ID
- `get_href() -> &str` - Get the file href/path
- `get_title() -> Option<&str>` - Get the file title
- `get_content() -> &str` - Get the file content as string
- `get_media_type() -> &str` - Get the MIME type
- `get_html_bytes() -> &[u8]` - Get raw HTML content as bytes
- `is_html() -> bool` - Check if the file is HTML
- `get_parsable_html() -> Option<String>` - Get parsable HTML content

### `TableOfContents`

Represents the table of contents.

#### Methods

- `get_entries() -> &Vec<TocEntry>` - Get all TOC entries
- `get_entry_count() -> usize` - Get the number of TOC entries

### `TocEntry`

Represents an entry in the table of contents.

#### Methods

- `get_title() -> &str` - Get the entry title
- `get_href() -> &str` - Get the entry href/link
- `get_level() -> u32` - Get the nesting level

## Running Examples

The library includes example code demonstrating various use cases:

```bash
# Run the basic usage example
cargo run --example basic_usage

# Run tests
cargo test
```

## Dependencies

- `chrono` - Date and time handling
- `uuid` - UUID generation and parsing
- `zip` - ZIP file handling (EPUB files are ZIP archives)
- `regex` - Regular expression support
- `serde` - Serialization framework
- `serde-xml-rs` - XML parsing

## Supported EPUB Features

- ✅ EPUB 2.0 and 3.0 formats
- ✅ OCF (Open Container Format) parsing
- ✅ OPF (Open Packaging Format) metadata extraction
- ✅ Navigation document parsing
- ✅ NCX (Navigation Control XML) support
- ✅ HTML content extraction
- ✅ Chapter organization and grouping

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Changelog

### 0.1.0
- Initial release
- Basic EPUB parsing functionality
- Metadata extraction
- Chapter and file organization
- Table of contents generation