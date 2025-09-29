# epubie-lib

⚠️ **This library is opinionated.**

If you want to poke around in the guts of the EPUB spec, this isn’t the crate for you.  
If you want to treat EPUBs as *books*—with chapters, files, and metadata, not a tangle of XML and ZIP internals—welcome! That’s exactly what `epubie-lib` is for.

---

## Why epubie-lib?

Most EPUB libraries try to expose every detail of the spec.  
This one doesn’t. It’s designed for *readers*, *writers*, and *builders* who just want to get at the stuff that matters:

- **What’s the title?**
- **Who wrote it?**
- **What are the chapters?**
- **What’s in each chapter?**
- **Can I get the HTML?**

You get a clean, high-level API that thinks like a book lover, not a standards committee.

---

## Philosophy

Think of it this way:

- 📘 **A `Chapter`** is a name + a list of files that make up that part of the book.
- 📄 **A `File`** might be HTML, CSS, or anything else—but HTML gets special treatment.
- 📚 **The Table of Contents** is a structured tree, not a raw navMap or NCX dump.

It’s not “technically correct” in the pedantic sense.  
It’s *usefully* correct.

---

## Features

- ✅ Parse EPUB metadata (title, author, language, etc.)
- ✅ Extract chapters and their content
- ✅ Generate a structured table of contents
- ✅ Access individual files (HTML, images, etc.)
- ✅ HTML content parsing and extraction
- ✅ Supports and 3.0

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
epubie-lib = "0.1.0"
```

---

## Quick Start

Here’s what working with an EPUB *should* feel like:

```rust
use epubie_lib::Epub;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let epub = Epub::new("path/to/your/book.epub".to_string())?;

    println!("Title: {}", epub.get_title());
    println!("Author: {}", epub.get_creator());

    for (i, chapter) in epub.get_chapters().iter().enumerate() {
        println!("Chapter {}: {}", i + 1, chapter.get_title());
        for file in chapter.get_files() {
            if file.is_html() {
                println!("  HTML content: {} bytes", file.get_html_bytes().len());
            }
        }
    }

    Ok(())
}
```

---

## Practical Examples

### Get the Metadata

```rust
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

### List Chapters and Their Files

```rust
let epub = Epub::new("book.epub".to_string())?;
println!("Total chapters: {}", epub.get_chapter_count());
for (i, chapter) in epub.get_chapters().iter().enumerate() {
    println!("Chapter {}: {}", i + 1, chapter.get_title());
    println!("  Files: {}", chapter.get_file_count());
    for file in chapter.get_files() {
        println!("    - {} ({})", file.get_title().unwrap_or("Untitled"), file.get_href());
    }
}
```

### Table of Contents, the Way You Want It

```rust
let epub = Epub::new("book.epub".to_string())?;
let toc = epub.get_table_of_contents();
println!("Table of Contents ({} entries):", toc.get_entry_count());
for entry in toc.get_entries() {
    let indent = "  ".repeat(entry.get_level() as usize);
    println!("{}{} -> {}", indent, entry.get_title(), entry.get_href());
}
```

### Access All Files (HTML, Images, etc.)

```rust
let epub = Epub::new("book.epub".to_string())?;
for file in epub.get_all_files() {
    println!("File: {} ({})", file.get_href(), file.get_media_type());
    if file.is_html() {
        let html_content = file.get_html_bytes();
        println!("  HTML size: {} bytes", html_content.len());
        if let Some(parsed_html) = file.get_parsable_html() {
            println!("  Parsed HTML available");
        }
    }
}
```

---

## API At a Glance

### `Epub`

- `new(file_path: String) -> Result<Epub, Box<dyn std::error::Error>>`
- `get_title() -> &str`
- `get_creator() -> &str`
- `get_language() -> &str`
- `get_identifier() -> &str`
- `get_date() -> &str`
- `get_publisher() -> Option<String>`
- `get_description() -> Option<String>`
- `get_rights() -> Option<String>`
- `get_cover() -> Option<String>`
- `get_tags() -> Option<Vec<String>>`
- `get_chapters() -> &Vec<Chapter>`
- `get_chapter_count() -> usize`
- `get_table_of_contents() -> &TableOfContents`
- `get_all_files() -> &Vec<EpubFile>`
- `get_file_count() -> usize`

### `Chapter`

- `get_title() -> &str`
- `get_files() -> &Vec<EpubFile>`
- `get_file_count() -> usize`

### `EpubFile`

- `get_id() -> &str`
- `get_href() -> &str`
- `get_title() -> Option<&str>`
- `get_content() -> &str`
- `get_media_type() -> &str`
- `get_html_bytes() -> &[u8]`
- `is_html() -> bool`
- `get_parsable_html() -> Option<String>`

### `TableOfContents`

- `get_entries() -> &Vec<TocEntry>`
- `get_entry_count() -> usize`

### `TocEntry`

- `get_title() -> &str`
- `get_href() -> &str`
- `get_level() -> u32`

---

## Running Examples

Want to see it in action?  
Run the included examples:

```bash
cargo run --example basic_usage
```

Or run the tests:

```bash
cargo test
```

---

## Under the Hood

- `chrono` — Date and time
- `uuid` — UUIDs
- `zip` — ZIP file handling (EPUBs are ZIPs)
- `regex` — Regular expressions
- `serde` — Serialization
- `serde-xml-rs` — XML parsing

---

## Supported EPUB Features

- ✅ EPUB 2.0 and 3.0
- ✅ OCF (Open Container Format)
- ✅ OPF (Open Packaging Format) metadata
- ✅ Navigation document parsing
- ✅ NCX (Navigation Control XML)
- ✅ HTML content extraction
- ✅ Chapter grouping

---

## Contributing

Pull requests are welcome!  
If you want to make this library better, open an issue or PR.

---

## Changelog

### 0.1.0

- Initial release
- Basic EPUB parsing
- Metadata extraction
- Chapter and file organization
- Table of contents generation

---