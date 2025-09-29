use epubie_lib::Epub;
use std::fs;

fn main() {
    let epub_path = "./example-files/wsw.epub";

    println!("Hello, World!");
    
    let epub = match Epub::new(epub_path.to_string()) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error parsing EPUB: {}", err);
            return;
        }
    };

    // Print basic metadata
    if let Some(title) = epub.get_title() {
        println!("Title: {}", title);
    }
    if let Some(creator) = epub.get_creator() {
        println!("Creator: {}", creator);
    }

    // Print chapters and files
    for chapter in epub.get_chapters() {
        println!("Chapter: {}", chapter.get_title());
        for file in chapter.get_files() {
            println!("  - {}", file.get_href());
        }
    }

    // Print content of first file (usually titlepage.xhtml)
    if let Some(first_file) = epub.get_chapters()
                                  .get(0)
                                  .and_then(|ch| ch.get_files().get(0)) {
        if first_file.is_html() {
            println!("\nContent of {}:", first_file.get_href());
            let bytes = first_file.get_html_bytes();
            if let Ok(content) = std::str::from_utf8(&bytes) {
                println!("{}", content);
            }
        }
    }
}
