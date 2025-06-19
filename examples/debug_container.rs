use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

fn main() {
    let epub_path = "./example-files/iia.epub";

    println!("Opening EPUB file: {}", epub_path);

    match File::open(epub_path) {
        Ok(file) => {
            match ZipArchive::new(file) {
                Ok(mut archive) => {
                    println!("Successfully opened ZIP archive");

                    // Try to read META-INF/container.xml
                    match archive.by_name("META-INF/container.xml") {
                        Ok(mut container_file) => {
                            let mut xml = String::new();
                            match container_file.read_to_string(&mut xml) {
                                Ok(_) => {
                                    println!("\n=== container.xml content ===");
                                    println!("{}", xml);

                                    println!("\n=== Container XML length: {} chars ===", xml.len());
                                }
                                Err(e) => {
                                    println!("Failed to read container.xml: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to find META-INF/container.xml: {}", e);
                        }
                    }

                    // Let's also check what the content.opf looks like
                    match archive.by_name("content.opf") {
                        Ok(mut opf_file) => {
                            let mut xml = String::new();
                            match opf_file.read_to_string(&mut xml) {
                                Ok(_) => {
                                    println!("\n=== content.opf content (first 1000 chars) ===");
                                    println!("{}", &xml[..xml.len().min(1000)]);
                                    if xml.len() > 1000 {
                                        println!(
                                            "... (truncated, total length: {} chars)",
                                            xml.len()
                                        );
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to read content.opf: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to find content.opf: {}", e);
                        }
                    }

                    // Let's also try OEBPS/content.opf
                    match archive.by_name("OEBPS/content.opf") {
                        Ok(mut opf_file) => {
                            let mut xml = String::new();
                            match opf_file.read_to_string(&mut xml) {
                                Ok(_) => {
                                    println!(
                                        "\n=== OEBPS/content.opf content (first 1000 chars) ==="
                                    );
                                    println!("{}", &xml[..xml.len().min(1000)]);
                                    if xml.len() > 1000 {
                                        println!(
                                            "... (truncated, total length: {} chars)",
                                            xml.len()
                                        );
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to read OEBPS/content.opf: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to find OEBPS/content.opf: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to open as ZIP archive: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to open file: {}", e);
        }
    }
}
