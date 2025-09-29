use epubie_lib::Epub;
use serde::Serialize;
use wasm_bindgen::prelude::*;

// Set panic hook for better error messages in the console
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct EpubWasm {
    epub: Epub,
}

#[derive(Serialize)]
struct FileData<'a> {
    title: Option<&'a str>,
    href: &'a str,
}

#[derive(Serialize)]
struct ChapterData<'a> {
    title: &'a str,
    html_files: Vec<FileData<'a>>,
}

#[wasm_bindgen]
impl EpubWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<EpubWasm, JsValue> {
        let epub =
            Epub::from_bytes(bytes.to_vec()).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(EpubWasm { epub })
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Option<String> {
        self.epub.get_title().map(|s| s.to_string())
    }

    #[wasm_bindgen(getter)]
    pub fn creator(&self) -> Option<String> {
        self.epub.get_creator().map(|s| s.to_string())
    }

    #[wasm_bindgen(js_name = getChapters)]
    pub fn get_chapters(&self) -> Result<JsValue, JsValue> {
        let chapters_data: Vec<ChapterData> = self
            .epub
            .get_chapters()
            .iter()
            .map(|c| {
                let html_files: Vec<FileData> = c
                    .get_files()
                    .iter()
                    .filter(|f| f.is_html())
                    .map(|f| FileData {
                        title: f.get_title(),
                        href: f.get_href(),
                    })
                    .collect();

                ChapterData {
                    title: c.get_title(),
                    html_files,
                }
            })
            .collect();

        serde_wasm_bindgen::to_value(&chapters_data).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = getFileContent)]
    pub fn get_file_content(&self, href: &str) -> Option<Vec<u8>> {
        self.epub
            .get_all_files()
            .iter()
            .find(|f| f.get_href() == href)
            .map(|f| f.get_content().as_bytes().to_vec())
    }
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
