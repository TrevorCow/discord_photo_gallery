use std::fs;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::website_builder::builder::gallery_page_info::GalleryPageInfo;

const WEBSITE_RESOURCE_GALLERY_TEMPLATE_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/website_files/gallery_template.html"));

static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("html_template", WEBSITE_RESOURCE_GALLERY_TEMPLATE_HTML).expect("Error registering gallery html template");
    handlebars
});

pub mod gallery_page_info {
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct GalleryPageInfo {
        pub(crate) page_title: String,

        pub(crate) galleries: Vec<Gallery>,

        pub(crate) guild_built_from: String,
        pub(crate) page_built_time: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Gallery {
        pub(crate) gallery_title: String,
        pub(crate) gallery_picture_infos: Vec<GalleryPictureInfo>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GalleryPictureInfo {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) description: Option<String>,
        pub(crate) discord_url: String,
        pub(crate) thumbnail_url: String,
    }
}

pub struct RenderedPage(pub(crate) String);

pub fn render_page(gallery_page_info: &GalleryPageInfo) -> RenderedPage {
    let built_html = HANDLEBARS.render("html_template", &gallery_page_info).expect("Failed to render gallery page info");

    RenderedPage(built_html)
}