use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use image::ImageFormat;
use image::imageops::FilterType;
use once_cell::sync::Lazy;
use reqwest::Url;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder().build().unwrap()
});

pub async fn download_image<P: AsRef<Path>>(website_root: P, image_url: &str) -> String {
    let image_url = Url::from_str(image_url).unwrap();

    let url_path = image_url.path().trim_start_matches("/");

    let save_path = website_root.as_ref().to_path_buf().join(url_path);

    if !save_path.exists() {
        println!("Starting download: {:?}", save_path);
        let thumbnail_image = async {
            let response = CLIENT.get(image_url).send().await.unwrap(); // This can fail if we can't connect to the discord CDN, if we can't connect there isn't much reason continuing anyway
            let image_bytes = response.bytes().await.unwrap(); // I think we can just unwrap here, I *think* that the only was this panics is if we run out of memory or something else where the best option is to just panic
            let image = image::load_from_memory(&image_bytes).unwrap();

            image.resize(250, 250, FilterType::Triangle) // This size here is based off of the values in gallery-style.css .gallery{}
        }.await;

        fs::create_dir_all(save_path.parent().unwrap()).unwrap();

        match thumbnail_image.save_with_format(&save_path, ImageFormat::Jpeg) {
            Ok(_) => {
                println!("Successfully saved thumbnail `{}`", save_path.display())
            }
            Err(err) => {
                eprintln!("Error saving thumbnail `{}`: {}", save_path.display(), err)
            }
        };
    } else {
        println!("Skipping already saved thumbnail `{}`", save_path.display())
    }

    save_path.strip_prefix(website_root).unwrap().to_string_lossy().to_string()
}
