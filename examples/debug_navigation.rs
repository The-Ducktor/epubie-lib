use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

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

fn parse_opf_xml(xml: &str) -> Result<Package, Box<dyn std::error::Error>> {
    let package: Package = serde_xml_rs::from_str(xml)?;
    Ok(package)
}

fn main() {
    let epub_path = "./example-files/iia.epub";

    println!("Testing navigation and manifest for: {}", epub_path);

    match File::open(epub_path) {
        Ok(file) => {
            match ZipArchive::new(file) {
                Ok(mut archive) => {
                    println!("Successfully opened ZIP archive");

                    // Read and parse content.opf
                    match archive.by_name("content.opf") {
                        Ok(mut opf_file) => {
                            let mut xml = String::new();
                            match opf_file.read_to_string(&mut xml) {
                                Ok(_) => {
                                    match parse_opf_xml(&xml) {
                                        Ok(package) => {
                                            println!("\n=== MANIFEST ITEMS ===");
                                            for (i, item) in
                                                package.manifest.item.iter().enumerate()
                                            {
                                                println!(
                                                    "{}: id={}, href={}, media-type={}, properties={:?}",
                                                    i + 1,
                                                    item.id,
                                                    item.href,
                                                    item.media_type,
                                                    item.properties
                                                );
                                            }

                                            // Find navigation file
                                            let nav_item =
                                                package.manifest.item.iter().find(|item| {
                                                    item.properties
                                                        .as_ref()
                                                        .map_or(false, |props| {
                                                            props.contains("nav")
                                                        })
                                                });

                                            if let Some(nav) = nav_item {
                                                println!("\n=== NAVIGATION FILE FOUND ===");
                                                println!(
                                                    "Navigation file: {} ({})",
                                                    nav.href, nav.id
                                                );

                                                // List path variations we would try
                                                let path_variations = vec![
                                                    nav.href.clone(),
                                                    format!("EPUB/{}", nav.href),
                                                    format!("OEBPS/{}", nav.href),
                                                ];

                                                println!("\nPath variations to try:");
                                                for path in path_variations {
                                                    println!("  {}", path);
                                                }
                                            } else {
                                                println!("\n=== NO NAVIGATION FILE FOUND ===");
                                                println!(
                                                    "Looking for items with 'nav' in properties..."
                                                );
                                                for item in &package.manifest.item {
                                                    if let Some(props) = &item.properties {
                                                        println!(
                                                            "Item {}: properties = '{}'",
                                                            item.id, props
                                                        );
                                                    }
                                                }
                                            }

                                            println!("\n=== SPINE ITEMS ===");
                                            for (i, spine_item) in
                                                package.spine.itemref.iter().enumerate()
                                            {
                                                println!("{}: idref={}", i + 1, spine_item.idref);
                                            }
                                        }
                                        Err(e) => {
                                            println!("Failed to parse OPF: {}", e);
                                        }
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
