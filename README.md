# epubie-lib

⚠️ **This library is opinionated.**

If you want to poke around in the guts of the EPUB spec, this isn't the crate for you.  
If you want to treat EPUBs as *books*—with chapters, files, and metadata—welcome! That's what `epubie-lib` is for.

---

## Why epubie-lib?

Most EPUB libraries expose every detail of the spec.  
This one doesn't. It's designed for people who just want answers to simple questions:

- **What's the title?**
- **Who wrote it?**
- **What are the chapters?**
- **What's the HTML content?**

You get a clean API that thinks like a book lover, not a standards committee.

---

## Philosophy

- 📘 **A `Chapter`** is a name + files that make up that section
- 📄 **A `File`** might be HTML, CSS, or anything else—HTML gets special treatment
- 📚 **The Table of Contents** is a structured tree, not XML soup

The library intelligently groups files into chapters by analyzing:
- Navigation structure from the EPUB's table of contents
- File naming patterns (e.g., `chapter_1_part1` and `chapter_1_part2` belong together)
- Spine ordering

It's not "technically correct" in the pedantic sense—it's *usefully* correct.

---

## Features

- ✅ Parse EPUB 3.0 metadata (title, author, language, etc.)
- ✅ Extract chapters and their content
- ✅ Generate structured table of contents
- ✅ Access individual files (HTML, images, etc.)
- ✅ HTML content parsing
- ✅ Smart chapter grouping based on navigation structure
- ❌ EPUB 2.0 is **not supported** (uses EPUB 3 navigation format)
- 🦀 WebAssembly version available for JS/TS (not size-optimized, use at your own risk)

---

## Installation

```toml
[dependencies]
epubie-lib = "0.1.0"
```

---

## Quick Start

```rust
use epubie_lib::Epub;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let epub = Epub::new("path/to/book.epub".to_string())?;

    println!("Title: {}", epub.get_title());
    println!("Author: {}", epub.get_creator());

    for (i, chapter) in epub.get_chapters().iter().enumerate() {
        println!("Chapter {}: {}", i + 1, chapter.get_title());
        for file in chapter.get_files() {
            if file.is_html() {
                println!("  HTML: {} bytes", file.get_html_bytes().len());
            }
        }
    }

    Ok(())
}
```

---

## Examples

### Get Metadata

```rust
let epub = Epub::new("book.epub".to_string())?;
println!("Title: {}", epub.get_title());
println!("Author: {}", epub.get_creator());
println!("Language: {}", epub.get_language());
```

### List Chapters

```rust
for (i, chapter) in epub.get_chapters().iter().enumerate() {
    println!("Chapter {}: {}", i + 1, chapter.get_title());
    for file in chapter.get_files() {
        println!("  - {}", file.get_href());
    }
}
```

### Table of Contents

```rust
let toc = epub.get_table_of_contents();
for entry in toc.get_entries() {
    let indent = "  ".repeat(entry.get_level() as usize);
    println!("{}{} -> {}", indent, entry.get_title(), entry.get_href());
}
```

---

## WebAssembly for JS/TS

There's a WebAssembly build you can use from JavaScript/TypeScript.

**Why it exists:** It was faster to compile Rust to WASM than rewrite the whole thing in JS.  
**Why you might not want it:** The bundle size isn't reasonable for production. It's more for testing or if you really don't care about bundle size.

Use it if you need EPUB parsing in the browser and size isn't a concern. Otherwise, do this server-side.

---

## Key Types

**`Epub`** - The main book object  
**`Chapter`** - A section with a title and files  
**`EpubFile`** - An individual file (HTML, CSS, images, etc.)  
**`TableOfContents`** - Navigation structure  
**`TocEntry`** - A single TOC item with title, link, and nesting level

Check the code for full method signatures—they're self-explanatory.

---

## Run Examples

```bash
cargo run --example basic_usage
cargo test
```

---


## Contributing

Found a bug? Want EPUB 2 support? Open an issue or PR.

---

## License

MIT

---
