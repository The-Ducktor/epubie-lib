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

use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

use std::io::{Cursor, Read, Seek};
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
    title: Option<String>,
    #[serde(rename = "dc:creator", default)]
    creator: Option<Vec<String>>,
    #[serde(rename = "dc:language")]
    language: Option<String>,
    #[serde(rename = "dc:date")]
    date: Option<String>,
    #[serde(rename = "dc:description")]
    description: Option<String>,
    #[serde(rename = "dc:publisher")]
    publisher: Option<String>,
    #[serde(rename = "dc:rights")]
    rights: Option<String>,
    #[serde(rename = "dc:subject", default)]
    subject: Vec<String>,
    #[serde(rename = "meta", default)]
    meta: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
struct Meta {
    #[serde(rename = "@name")]
    name: Option<String>,
    #[serde(rename = "@content")]
    content: Option<String>,
    #[serde(rename = "@property")]
    property: Option<String>,
    #[serde(rename = "@refines")]
    refines: Option<String>,
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

/// Metadata structure containing all EPUB metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    title: Option<String>,
    creator: Vec<String>,
    language: Option<String>,
    identifier: String,
    date: Option<String>,
    publisher: Option<String>,
    description: Option<String>,
    rights: Option<String>,
    cover: Option<String>,
    tags: Vec<String>,
}

impl Metadata {
    pub fn new(
        title: Option<String>,
        creator: Vec<String>,
        language: Option<String>,
        identifier: String,
        date: Option<String>,
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
            tags: Vec::new(),
        }
    }

    pub fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn get_creators(&self) -> &[String] {
        &self.creator
    }

    pub fn get_language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    pub fn get_publisher(&self) -> Option<&str> {
        self.publisher.as_deref()
    }

    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn get_rights(&self) -> Option<&str> {
        self.rights.as_deref()
    }

    pub fn get_cover(&self) -> Option<&str> {
        self.cover.as_deref()
    }

    pub fn get_tags(&self) -> &[String] {
        &self.tags
    }
}

/// Main EPUB container that holds all parsed data
pub struct Epub {
    metadata: Metadata,
    chapters: Vec<Chapter>,
    table_of_contents: TableOfContents,
    all_files: Vec<EpubFile>,
    file_bytes: Vec<u8>,
}

impl Epub {
    /// Creates a new Epub instance by parsing the EPUB file from bytes
    ///
    /// # Arguments
    /// * `file_bytes` - Bytes of the EPUB file
    ///
    /// # Returns
    /// * `Result<Epub, Box<dyn Error>>` - Parsed EPUB or error
    pub fn new(file_bytes: Vec<u8>) -> Result<Epub, Box<dyn Error>> {
        let mut archive = ZipArchive::new(Cursor::new(&file_bytes))?;

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
            package.metadata.creator.clone().unwrap_or_default(),
            package.metadata.language.clone(),
            package
                .metadata
                .identifier
                .first()
                .cloned()
                .unwrap_or_default(),
            package.metadata.date.clone(),
        );

        // Set optional metadata fields
        metadata.description = package.metadata.description.clone();
        metadata.publisher = package.metadata.publisher.clone();
        metadata.rights = package.metadata.rights.clone();
        metadata.tags = package.metadata.subject.clone();

        // Find cover from meta tags - handle both EPUB 2 and 3 formats
        metadata.cover = Self::find_cover_id(&package);

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
            file_bytes,
        })
    }

    // Getter methods for accessing parsed data
    pub fn get_title(&self) -> Option<&str> {
        self.metadata.get_title()
    }

    pub fn get_creator(&self) -> Option<&str> {
        self.metadata.get_creators().first().map(|s| s.as_str())
    }

    pub fn get_creators(&self) -> &[String] {
        self.metadata.get_creators()
    }

    pub fn get_language(&self) -> Option<&str> {
        self.metadata.get_language()
    }

    pub fn get_identifier(&self) -> &str {
        self.metadata.get_identifier()
    }

    pub fn get_date(&self) -> Option<&str> {
        self.metadata.get_date()
    }

    pub fn get_publisher(&self) -> Option<&str> {
        self.metadata.get_publisher()
    }

    pub fn get_description(&self) -> Option<&str> {
        self.metadata.get_description()
    }

    pub fn get_rights(&self) -> Option<&str> {
        self.metadata.get_rights()
    }

    pub fn get_cover(&self) -> Option<&str> {
        self.metadata.get_cover()
    }

    pub fn get_tags(&self) -> &[String] {
        self.metadata.get_tags()
    }

    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get cover image as bytes
    pub fn get_cover_bytes(&self) -> Option<Vec<u8>> {
        let cover_id = self.metadata.cover.as_ref()?;

        // Open the EPUB file from bytes
        let mut archive = ZipArchive::new(Cursor::new(&self.file_bytes)).ok()?;

        // Read container.xml
        let mut xml = String::new();
        {
            let mut container_file = archive.by_name("META-INF/container.xml").ok()?;
            container_file.read_to_string(&mut xml).ok()?;
        }
        let container = parse_container_xml(&xml).ok()?;
        let opf_path = &container.rootfiles.rootfile[0].full_path;

        // Read OPF file
        let mut opf_xml = String::new();
        {
            let mut opf_file = archive.by_name(opf_path).ok()?;
            opf_file.read_to_string(&mut opf_xml).ok()?;
        }
        let package = parse_opf_xml(&opf_xml).ok()?;

        // Find the manifest item with the cover id
        let manifest_item = package
            .manifest
            .item
            .iter()
            .find(|item| &item.id == cover_id)?;

        let cover_href = &manifest_item.href;

        // Resolve the cover file path relative to the OPF directory
        let cover_path = Self::resolve_path(opf_path, cover_href);

        // Extract the cover file as bytes
        let mut buf = Vec::new();
        {
            let mut cover_file = archive.by_name(&cover_path).ok()?;
            cover_file.read_to_end(&mut buf).ok()?;
        }
        Some(buf)
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

    /// Find cover ID from metadata - handles both EPUB 2 and 3 formats
    fn find_cover_id(package: &Package) -> Option<String> {
        // EPUB 2: Look for meta with name="cover"
        for meta in &package.metadata.meta {
            if let (Some(name), Some(content)) = (&meta.name, &meta.content) {
                if name == "cover" {
                    return Some(content.clone());
                }
            }
        }

        // EPUB 3: Look for meta with property="cover-image"
        for meta in &package.metadata.meta {
            if let Some(property) = &meta.property {
                if property == "cover-image" {
                    if let Some(content) = &meta.content {
                        return Some(content.clone());
                    }
                    // Sometimes the ID is in the text content
                    if let Some(value) = &meta.value {
                        return Some(value.clone());
                    }
                }
            }
        }

        // Fallback: Look for manifest items with properties="cover-image"
        for item in &package.manifest.item {
            if let Some(properties) = &item.properties {
                if properties.contains("cover-image") {
                    return Some(item.id.clone());
                }
            }
        }

        None
    }

    /// Resolve a relative path against a base path
    fn resolve_path(base_path: &str, relative_path: &str) -> String {
        if let Some(slash_pos) = base_path.rfind('/') {
            format!("{}/{}", &base_path[..slash_pos], relative_path)
        } else {
            relative_path.to_string()
        }
    }

    fn parse_navigation(
        archive: &mut ZipArchive<impl Read + Seek>,
        package: &Package,
        opf_path: &str,
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut nav_titles = HashMap::new();

        // Find the navigation file in the manifest
        if let Some(nav_item) = package.manifest.item.iter().find(|item| {
            item.properties
                .as_ref()
                .map_or(false, |props| props.contains("nav"))
        }) {
            let nav_path = Self::resolve_path(opf_path, &nav_item.href);

            // Try to parse the navigation file
            if let Ok(mut nav_file) = archive.by_name(&nav_path) {
                let mut html = String::new();
                if nav_file.read_to_string(&mut html).is_ok() {
                    // Use regex to extract href and text from <a> tags
                    let pattern = r#"<a\s+href="([^"]+)"[^>]*>([^<]+)</a>"#;
                    if let Ok(re) = Regex::new(pattern) {
                        for cap in re.captures_iter(&html) {
                            if let (Some(href), Some(text)) = (cap.get(1), cap.get(2)) {
                                let href_str = href.as_str().to_string();
                                let text_str = text.as_str().trim().to_string();
                                nav_titles.insert(href_str, text_str);
                            }
                        }
                    }
                }
            }
        }

        Ok(nav_titles)
    }

    fn parse_all_files(
        archive: &mut ZipArchive<impl Read + Seek>,
        package: &Package,
        nav_titles: &HashMap<String, String>,
        opf_path: &str,
    ) -> Result<Vec<EpubFile>, Box<dyn Error>> {
        let mut files = Vec::new();

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

                let file_path = Self::resolve_path(opf_path, &manifest_item.href);

                if let Ok(mut file) = archive.by_name(&file_path) {
                    let mut content = String::new();
                    if file.read_to_string(&mut content).is_ok() {
                        let epub_file = EpubFile {
                            id: manifest_item.id.clone(),
                            href: manifest_item.href.clone(),
                            title: nav_titles.get(&manifest_item.href).cloned(),
                            content,
                            media_type: manifest_item.media_type.clone(),
                        };

                        files.push(epub_file);
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

        for itemref in &spine.itemref {
            if let Some(file) = file_map.get(&itemref.idref) {
                // Determine if this should start a new chapter
                let should_start_new_chapter = if current_chapter_files.is_empty() {
                    true
                } else {
                    // Start new chapter if:
                    // 1. The file has a title from navigation
                    // 2. The base name changes significantly
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
}

// Function to parse container.xml using serde-xml-rs
fn parse_container_xml(xml: &str) -> Result<Container, Box<dyn Error>> {
    let container: Container = serde_xml_rs::from_str(xml)?;
    Ok(container)
}

// Function to parse OPF file using serde-xml-rs
fn parse_opf_xml(xml: &str) -> Result<Package, Box<dyn Error>> {
    let package: Package = serde_xml_rs::from_str(xml)?;
    Ok(package)
}
