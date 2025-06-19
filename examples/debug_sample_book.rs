use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
struct Container {
    #[serde(rename = "rootfiles")]
    rootfiles: RootFiles,
}

#[derive(Debug, Deserialize)]
struct RootFiles {
    #[serde(rename = "rootfile")]
    rootfile: Vec<RootFile>,
}

#[derive(Debug, Deserialize)]
struct RootFile {
    #[serde(rename = "@full-path", default)]
    full_path: String,
    #[serde(rename = "@media-type", default)]
    media_type: String,
}

#[derive(Debug, Deserialize)]
struct Package {
    metadata: OpfMetadata,
    manifest: Manifest,
    spine: Spine,
}

#[derive(Debug, Deserialize)]
struct OpfMetadata {
    #[serde(rename = "dc:identifier", default)]
    identifier: Vec<String>,
    #[serde(rename = "dc:title")]
    title: String,
    #[serde(rename = "dc:creator")]
    creator: String,
    #[serde(rename = "dc:language")]
    language: String,
    #[serde(rename = "dc:date")]
    date: String,
    #[serde(rename = "dc:description")]
    description: Option<String>,
    #[serde(rename = "meta", default)]
    meta: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
struct Meta {
    #[serde(rename = "@name", default)]
    name: Option<String>,
    #[serde(rename = "@content", default)]
    content: Option<String>,
    #[serde(rename = "@property", default)]
    property: Option<String>,
    #[serde(rename = "$text", default)]
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "item", default)]
    item: Vec<ManifestItem>,
}

#[derive(Debug, Deserialize)]
struct ManifestItem {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "@media-type")]
    media_type: String,
    #[serde(rename = "@properties")]
    properties: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Spine {
    #[serde(rename = "itemref", default)]
    itemref: Vec<ItemRef>,
}

#[derive(Debug, Deserialize)]
struct ItemRef {
    #[serde(rename = "@idref")]
    idref: String,
}

fn parse_container_xml(xml: &str) -> Result<Container, Box<dyn std::error::Error>> {
    let container: Container = serde_xml_rs::from_str(xml)?;
    Ok(container)
}

fn parse_opf_xml(xml: &str) -> Result<Package, Box<dyn std::error::Error>> {
    let package: Package = serde_xml_rs::from_str(xml)?;
    Ok(package)
}

fn main() {
    let epub_path = "./example-files/sample-book.epub";

    println!("Debugging sample-book.epub structure: {}", epub_path);

    match File::open(epub_path) {
        Ok(file) => {
            match ZipArchive::new(file) {
                Ok(mut archive) => {
                    println!("Successfully opened ZIP archive");
                    println!("Number of files: {}", archive.len());

                    // List all files first
                    println!("\n=== ALL FILES IN ARCHIVE ===");
                    for i in 0..archive.len() {
                        if let Ok(file) = archive.by_index(i) {
                            println!("  {}: {}", i, file.name());
                        }
                    }

                    // Parse container.xml
                    match archive.by_name("META-INF/container.xml") {
                        Ok(mut container_file) => {
                            let mut xml = String::new();
                            match container_file.read_to_string(&mut xml) {
                                Ok(_) => {
                                    println!("\n=== CONTAINER.XML ===");
                                    println!("{}", xml);

                                    match parse_container_xml(&xml) {
                                        Ok(container) => {
                                            println!("\n=== PARSED CONTAINER ===");
                                            println!("{:#?}", container);

                                            if !container.rootfiles.rootfile.is_empty() {
                                                let opf_path =
                                                    &container.rootfiles.rootfile[0].full_path;
                                                println!("\nOPF path: {}", opf_path);

                                                // Check if OPF file exists at this path
                                                println!("Checking if OPF exists at: {}", opf_path);
                                            }
                                        }
                                        Err(e) => {
                                            println!("Failed to parse container.xml: {}", e);
                                        }
                                    }
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
