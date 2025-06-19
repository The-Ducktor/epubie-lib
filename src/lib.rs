pub mod epub;

pub use epub::Epub;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {

    use crate::epub::Epub;

    #[test]
    fn example_epub() {
        let path = "./example-files/iia.epub";

        match Epub::new(path.to_string()) {
            Ok(epub) => {
                for (i, entry) in epub.get_chapters().iter().enumerate() {
                    println!(c
                        "Chapter {}: {} ({} file{})",
                        i + 1,
                        entry.get_title(),
                        entry.get_file_count(),
                        if entry.get_file_count() == 1 { "" } else { "s" }
                    );

                    for (j, file) in entry.get_files().iter().enumerate() {
                        println!(
                            "  File {}: {} ({})",
                            j + 1,
                            file.get_title().unwrap_or("No title"),
                            file.get_href()
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("Error parsing EPUB: {}", e);
            }
        }
    }
}
