//! EPUB Parser Library
//!
//! This library provides functionality to parse EPUB files and extract:
//! - Metadata (title, creator, language, identifier, date, description, cover)
//! - Chapter information with titles from navigation file
//! - Complete HTML content from all XHTML files for external parsing
//! - Proper handling of Dublin Core metadata elements
//! - Navigation file parsing to extract actual chapter titles
//!
//! The parser handles the standard EPUB 3.0 format including:
//! - META-INF/container.xml parsing to locate the OPF file
//! - OPF (Open Packaging Format) file parsing for metadata and manifest
//! - Navigation file (nav.xhtml) parsing for chapter titles
//! - Full XHTML content extraction for use with external HTML parsers
//! - Fallback to manifest IDs when navigation titles are not available
//!
//! ## HTML Content Access
//!
//! Each `EpubFile` contains the complete XHTML content which can be parsed using
//! external HTML parsing libraries like `scraper`, `select`, or `html5ever`:
//!
//! ```rust,ignore
//! // Example using the scraper crate
//! use scraper::{Html, Selector};
//!
//! for chapter in epub.get_chapters() {
//!     for file in chapter.get_files() {
//!         let document = Html::parse_document(file.get_parsable_html());
//!         let selector = Selector::parse("p").unwrap();
//!
//!         for element in document.select(&selector) {
//!             println!("Paragraph: {}", element.text().collect::<String>());
//!         }
//!     }
//! }
//! ```

use regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::Read;
use zip::read::ZipArchive;

/// Represents a single file within an EPUB
#[derive(Debug, Clone)]
pub struct EpubFile {
    pub id: String,
    pub href: String,
    pub title: Option<String>,
    pub content: String,
    pub media_type: String,
}

impl EpubFile {
    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_href(&self) -> &str {
        &self.href
    }

    pub fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_media_type(&self) -> &str {
        &self.media_type
    }

    /// Get HTML content as bytes for parsing with external libraries
    pub fn get_html_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }

    /// Check if this file contains HTML content
    pub fn is_html(&self) -> bool {
        self.media_type == "application/xhtml+xml"
    }

    /// Get the HTML content ready for parsing with external HTML parsers
    pub fn get_parsable_html(&self) -> &str {
        &self.content
    }
}

/// Represents a chapter that can contain multiple files
pub struct Chapter {
    title: String,
    files: Vec<EpubFile>,
}

impl Chapter {
    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_files(&self) -> &[EpubFile] {
        &self.files
    }

    pub fn get_file_count(&self) -> usize {
        self.files.len()
    }
}

/// Table of Contents entry
#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub href: String,
    pub level: usize,
}

impl TocEntry {
    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_href(&self) -> &str {
        &self.href
    }

    pub fn get_level(&self) -> usize {
        self.level
    }
}

/// Complete Table of Contents
pub struct TableOfContents {
    entries: Vec<TocEntry>,
}

impl TableOfContents {
    pub fn new() -> Self {
        TableOfContents {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, title: String, href: String, level: usize) {
        self.entries.push(TocEntry { title, href, level });
    }

    pub fn get_entries(&self) -> &[TocEntry] {
        &self.entries
    }

    pub fn get_entry_count(&self) -> usize {
        self.entries.len()
    }
}

// Structs for parsing container.xml
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

// Structs for parsing OPF file
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
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "content")]
    content: Option<String>,
    #[serde(rename = "property")]
    property: Option<String>,
    #[serde(rename = "$text")]
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "item")]
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
    #[serde(rename = "itemref")]
    itemref: Vec<ItemRef>,
}

#[derive(Debug, Deserialize)]
struct ItemRef {
    #[serde(rename = "@idref")]
    idref: String,
}

// Define the metadata structure
struct Metadata {
    title: String,
    creator: String,
    language: String,
    identifier: String,
    date: String,
    publisher: Option<String>,
    description: Option<String>,
    rights: Option<String>,
    cover: Option<String>,
    tags: Vec<String>,
}

impl Metadata {
    pub fn new(
        title: String,
        creator: String,
        language: String,
        identifier: String,
        date: String,
    ) -> Self {
        Metadata {
            title,
            creator,
            language,
            identifier,
            date,
            publisher: None,
            description: None,
            rights: None,
            cover: None,
            tags: vec![],
        }
    }
}

/// Main EPUB container that holds all parsed data
pub struct Epub {
    metadata: Metadata,
    chapters: Vec<Chapter>,
    table_of_contents: TableOfContents,
    all_files: Vec<EpubFile>,
}

impl Epub {
    /// Creates a new Epub instance by parsing the EPUB file at the given path
    ///
    /// # Arguments
    /// * `file_path` - Path to the EPUB file
    ///
    /// # Returns
    /// * `Result<Epub, Box<dyn error::Error>>` - Parsed EPUB or error
    pub fn new(file_path: String) -> Result<Epub, Box<dyn error::Error>> {
        let file = File::open(file_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Read and parse META-INF/container.xml
        let container = {
            let mut container_file = archive.by_name("META-INF/container.xml")?;
            let mut xml = String::new();
            container_file.read_to_string(&mut xml)?;
            parse_container_xml(&xml)?
        };

        // Get the OPF path and parse OPF file
        let opf_path = &container.rootfiles.rootfile[0].full_path;
        let package = {
            let mut opf_file = archive.by_name(&opf_path)?;
            let mut xml = String::new();
            opf_file.read_to_string(&mut xml)?;
            parse_opf_xml(&xml)?
        };

        // Parse navigation file to get chapter titles first
        let nav_titles = Self::parse_navigation(&mut archive, &package, &opf_path)?;

        // Extract metadata from OPF
        let mut metadata = Metadata::new(
            package.metadata.title.clone(),
            package.metadata.creator.clone(),
            package.metadata.language.clone(),
            package
                .metadata
                .identifier
                .first()
                .unwrap_or(&String::new())
                .clone(),
            package.metadata.date.clone(),
        );

        // Set optional metadata fields
        metadata.description = package.metadata.description.clone();

        // Find cover from meta tags
        for meta in &package.metadata.meta {
            if let (Some(name), Some(content)) = (&meta.name, &meta.content) {
                if name == "cover" {
                    metadata.cover = Some(content.clone());
                }
            }
        }

        // Parse all XHTML files and create EpubFile objects
        let all_files = Self::parse_all_files(&mut archive, &package, &nav_titles, &opf_path)?;

        // Create table of contents from navigation
        let table_of_contents = Self::create_table_of_contents(&nav_titles, &all_files);

        // Group files into chapters
        let chapters = Self::group_files_into_chapters(&all_files, &package.spine);

        Ok(Epub {
            metadata,
            chapters,
            table_of_contents,
            all_files,
        })
    }

    // Getter methods for accessing parsed data
    pub fn get_title(&self) -> &str {
        &self.metadata.title
    }

    pub fn get_creator(&self) -> &str {
        &self.metadata.creator
    }

    pub fn get_language(&self) -> &str {
        &self.metadata.language
    }

    pub fn get_identifier(&self) -> &str {
        &self.metadata.identifier
    }

    pub fn get_date(&self) -> &str {
        &self.metadata.date
    }

    pub fn get_publisher(&self) -> Option<&str> {
        self.metadata.publisher.as_deref()
    }

    pub fn get_description(&self) -> Option<&str> {
        self.metadata.description.as_deref()
    }

    pub fn get_rights(&self) -> Option<&str> {
        self.metadata.rights.as_deref()
    }

    pub fn get_cover(&self) -> Option<&str> {
        self.metadata.cover.as_deref()
    }

    pub fn get_tags(&self) -> &[String] {
        &self.metadata.tags
    }

    pub fn get_chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    pub fn get_chapter_count(&self) -> usize {
        self.chapters.len()
    }

    pub fn get_table_of_contents(&self) -> &TableOfContents {
        &self.table_of_contents
    }

    pub fn get_all_files(&self) -> &[EpubFile] {
        &self.all_files
    }

    pub fn get_file_count(&self) -> usize {
        self.all_files.len()
    }

    fn parse_navigation(
        archive: &mut ZipArchive<File>,
        package: &Package,
        opf_path: &str,
    ) -> Result<HashMap<String, String>, Box<dyn error::Error>> {
        let mut nav_titles = HashMap::new();

        // Find the navigation file in the manifest
        if let Some(nav_item) = package.manifest.item.iter().find(|item| {
            item.properties
                .as_ref()
                .map_or(false, |props| props.contains("nav"))
        }) {
            // Resolve the navigation file path relative to the OPF directory
            let opf_dir = if let Some(slash_pos) = opf_path.rfind('/') {
                &opf_path[..slash_pos + 1] // Include the trailing slash
            } else {
                "" // OPF is at root level
            };
            let nav_path = format!("{}{}", opf_dir, nav_item.href);

            // Try to parse the navigation file
            match archive.by_name(&nav_path) {
                Ok(mut nav_file) => {
                    let mut html = String::new();
                    nav_file.read_to_string(&mut html)?;

                    // Use simple regex pattern to extract href and text from <a> tags
                    // Pattern: <a href="..." ...>TEXT</a>
                    let pattern = r#"<a\s+href="([^"]+)"[^>]*>([^<]+)</a>"#;
                    if let Ok(re) = regex::Regex::new(pattern) {
                        for cap in re.captures_iter(&html) {
                            if let (Some(href), Some(text)) = (cap.get(1), cap.get(2)) {
                                let href_str = href.as_str().to_string();
                                let text_str = text.as_str().trim().to_string();
                                nav_titles.insert(href_str, text_str);
                            }
                        }
                    }
                }
                Err(_) => {
                    // Navigation file not found or couldn't be read, continue without titles
                }
            }
        }

        Ok(nav_titles)
    }

    fn parse_all_files(
        archive: &mut ZipArchive<File>,
        package: &Package,
        nav_titles: &HashMap<String, String>,
        opf_path: &str,
    ) -> Result<Vec<EpubFile>, Box<dyn error::Error>> {
        let mut files = Vec::new();

        // Determine the OPF directory for resolving relative paths
        let opf_dir = if let Some(slash_pos) = opf_path.rfind('/') {
            &opf_path[..slash_pos + 1] // Include the trailing slash
        } else {
            "" // OPF is at root level
        };

        for manifest_item in &package.manifest.item {
            if manifest_item.media_type == "application/xhtml+xml" {
                // Skip navigation files
                let is_nav = manifest_item
                    .properties
                    .as_ref()
                    .map_or(false, |props| props.contains("nav"));

                if is_nav {
                    continue;
                }

                // Resolve the file path relative to the OPF directory
                let file_path = format!("{}{}", opf_dir, manifest_item.href);

                match archive.by_name(&file_path) {
                    Ok(mut file) => {
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;

                        let epub_file = EpubFile {
                            id: manifest_item.id.clone(),
                            href: manifest_item.href.clone(),
                            title: nav_titles.get(&manifest_item.href).cloned(),
                            content,
                            media_type: manifest_item.media_type.clone(),
                        };

                        files.push(epub_file);
                    }
                    Err(_) => {
                        // File not found or couldn't be read, skip it
                        continue;
                    }
                }
            }
        }

        Ok(files)
    }

    fn create_table_of_contents(
        _nav_titles: &HashMap<String, String>,
        all_files: &[EpubFile],
    ) -> TableOfContents {
        let mut toc = TableOfContents::new();

        // Add entries for all content files in spine order
        for file in all_files {
            let title = file.title.clone().unwrap_or_else(|| file.id.clone());
            toc.add_entry(title, file.href.clone(), 0);
        }

        toc
    }

    fn group_files_into_chapters(all_files: &[EpubFile], spine: &Spine) -> Vec<Chapter> {
        let mut chapters = Vec::new();
        let mut current_chapter_files = Vec::new();
        let mut current_chapter_title = String::new();

        // Create a map from ID to file for easy lookup
        let file_map: HashMap<String, &EpubFile> = all_files
            .iter()
            .map(|file| (file.id.clone(), file))
            .collect();

        for (_index, itemref) in spine.itemref.iter().enumerate() {
            if let Some(file) = file_map.get(&itemref.idref) {
                // Determine if this should start a new chapter
                let should_start_new_chapter = if current_chapter_files.is_empty() {
                    true
                } else {
                    // Start new chapter if:
                    // 1. The file has a title from navigation
                    // 2. The base name changes (e.g., chapter_1 vs chapter_2)
                    file.title.is_some()
                        && !Self::files_belong_to_same_chapter(&current_chapter_files[0], file)
                };

                if should_start_new_chapter && !current_chapter_files.is_empty() {
                    // Finish current chapter
                    let chapter = Chapter {
                        title: current_chapter_title.clone(),
                        files: current_chapter_files.clone(),
                    };
                    chapters.push(chapter);
                    current_chapter_files.clear();
                }

                if current_chapter_files.is_empty() {
                    // Starting a new chapter
                    current_chapter_title = file.title.clone().unwrap_or_else(|| file.id.clone());
                }

                current_chapter_files.push((*file).clone());
            }
        }

        // Add the last chapter if there are remaining files
        if !current_chapter_files.is_empty() {
            let chapter = Chapter {
                title: current_chapter_title,
                files: current_chapter_files,
            };
            chapters.push(chapter);
        }

        chapters
    }

    fn files_belong_to_same_chapter(file1: &EpubFile, file2: &EpubFile) -> bool {
        // Extract base chapter name (everything before the last underscore and number)
        let base1 = Self::extract_chapter_base(&file1.id);
        let base2 = Self::extract_chapter_base(&file2.id);
        base1 == base2
    }

    fn extract_chapter_base(id: &str) -> String {
        // For IDs like "chapter_4_part1", extract "chapter_4"
        // For IDs like "chapter_1", extract "chapter_1"
        if let Some(last_underscore) = id.rfind('_') {
            let after_underscore = &id[last_underscore + 1..];
            // If what comes after the underscore starts with "part",
            // return everything before the "_part" part
            if after_underscore.starts_with("part") {
                return id[..last_underscore].to_string();
            }
        }
        id.to_string()
    }

    fn get_zip_archive(file_path: &str) -> Result<ZipArchive<File>, Box<dyn error::Error>> {
        let file = File::open(file_path)?;
        let archive = ZipArchive::new(file)?;
        Ok(archive)
    }
}

// Function to parse container.xml using serde-xml-rs
fn parse_container_xml(xml: &str) -> Result<Container, Box<dyn std::error::Error>> {
    let container: Container = serde_xml_rs::from_str(xml)?;
    Ok(container)
}

// Function to parse OPF file using serde-xml-rs
fn parse_opf_xml(xml: &str) -> Result<Package, Box<dyn std::error::Error>> {
    let package: Package = serde_xml_rs::from_str(xml)?;
    Ok(package)
}
