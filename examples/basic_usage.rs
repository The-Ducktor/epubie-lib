use epubie_lib::Epub;

fn main() {
    // Example of how to use the epubie-lib library
    let epub_path = "./example-files/iia.epub";

    println!("Attempting to parse EPUB: {}", epub_path);

    match Epub::new(epub_path.to_string()) {
        Ok(epub) => {
            // Display basic metadata
            println!("\n=== EPUB Metadata ===");
            println!("Title: {}", epub.get_title());
            println!("Creator: {}", epub.get_creator());
            println!("Language: {}", epub.get_language());
            println!("Identifier: {}", epub.get_identifier());
            println!("Date: {}", epub.get_date());

            if let Some(publisher) = epub.get_publisher() {
                println!("Publisher: {}", publisher);
            }

            if let Some(description) = epub.get_description() {
                println!("Description: {}", description);
            }

            // Display chapter information
            println!("\n=== Chapters ({}) ===", epub.get_chapter_count());
            for (i, chapter) in epub.get_chapters().iter().enumerate() {
                println!(
                    "Chapter {}: {} ({} file{})",
                    i + 1,
                    chapter.get_title(),
                    chapter.get_file_count(),
                    if chapter.get_file_count() == 1 {
                        ""
                    } else {
                        "s"
                    }
                );

                // Display files in each chapter
                for (j, file) in chapter.get_files().iter().enumerate() {
                    println!(
                        "  File {}: {} ({})",
                        j + 1,
                        file.get_title().unwrap_or("No title"),
                        file.get_href()
                    );

                    // Show if it's an HTML file
                    if file.is_html() {
                        println!("    [HTML file - {} bytes]", file.get_html_bytes().len());
                    }
                }
            }

            // Display table of contents
            let toc = epub.get_table_of_contents();
            println!(
                "\n=== Table of Contents ({} entries) ===",
                toc.get_entry_count()
            );
            for (i, entry) in toc.get_entries().iter().enumerate() {
                let indent = "  ".repeat(entry.get_level() as usize);
                println!(
                    "{}{}: {} ({})",
                    indent,
                    i + 1,
                    entry.get_title(),
                    entry.get_href()
                );
            }

            // Display total file count
            println!("\n=== File Summary ===");
            println!("Total files: {}", epub.get_file_count());
            println!("All files:");
            for (i, file) in epub.get_all_files().iter().enumerate() {
                println!(
                    "  {}: {} ({}) - {}",
                    i + 1,
                    file.get_title().unwrap_or("No title"),
                    file.get_href(),
                    file.get_media_type()
                );
            }
        }
        Err(e) => {
            eprintln!("Error parsing EPUB: {}", e);
            eprintln!("Make sure the file exists and is a valid EPUB file.");
        }
    }
}
