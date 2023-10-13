use std::fs;
use std::path::{Path, PathBuf};

use crate::website::builder::RenderedPage;

pub mod builder;

const WEBSITE_RESOURCE_GALLERY_JS_SRC: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/website_files/resources/gallery.js"));
const WEBSITE_RESOURCE_GALLERY_STYLES_CSS_SRC: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/website_files/resources/gallery-style.css"));

pub fn write_whole_website_directory<P: AsRef<Path>>(path: P, rendered_page: &RenderedPage) {
    let website_folder_path = PathBuf::from(path.as_ref());
    let website_resources_path = website_folder_path.join("resources");

    fs::create_dir_all(&website_folder_path).expect("Failed to create website folder");
    fs::create_dir_all(&website_resources_path).expect("Failed to create website resources folder");
    fs::write(website_resources_path.join("gallery.js"), WEBSITE_RESOURCE_GALLERY_JS_SRC).unwrap();
    fs::write(website_resources_path.join("gallery-style.css"), WEBSITE_RESOURCE_GALLERY_STYLES_CSS_SRC).unwrap();
    fs::write(website_folder_path.join("index.html"), &rendered_page.0).unwrap();
}