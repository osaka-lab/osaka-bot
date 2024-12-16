use bsky_sdk::{api::{types::Union, app::bsky::embed::images, app::bsky::feed::post}, BskyAgent};
use glob::glob;
use std::{fs, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use rand::{seq::SliceRandom, thread_rng};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct Images {
    path: String,
    uploaded_path: String,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub path: PathBuf,
    pub toml_path: Option<PathBuf>,
    pub credit: Option<String>
}

#[derive(Deserialize)]
struct CreditToml {
    credit: String
}

impl Default for Images {
    fn default() -> Self {
        let path = "./images".to_string();
        let uploaded_path = "./uploaded".to_string();

        if let Err(err) = fs::create_dir_all(&path) {
            panic!("Failed to create folder '{}': {}", &path, err);   
        }

        if let Err(err) = fs::create_dir_all(&uploaded_path) {
            panic!("Failed to create folder '{}': {}", &uploaded_path, err);   
        }

        Self {
            path,
            uploaded_path
        }
    }
}

impl Images {
    pub fn move_files(&self, image: &Image) {
        let path = &image.path;
        let file_name = path.file_name().unwrap();

        let dist = format!("{}/{}", self.uploaded_path, file_name.to_string_lossy());
        if let Err(e) = std::fs::rename(&path, &dist) {
            eprintln!("Failed to move file '{:?}' to '{:?}': '{}'", path, dist, e);
        }

        let toml_path = &image.toml_path;

        if toml_path.is_some() {
            let path = toml_path.as_ref().unwrap();
            let file_name = path.file_name().unwrap();
    
            let dist = format!("{}/{}", self.uploaded_path, file_name.to_string_lossy());
            if let Err(e) = std::fs::rename(&path, &dist) {
                eprintln!("Failed to move file '{:?}' to '{:?}': '{}'", path, dist, e);
            }
        }

    }

    pub fn get_random_image(&mut self) -> Option<Image> {
        let png = &format!("{}/*.png", self.path);
        let jpg = &format!("{}/*.jpg", self.path);
        let jpeg = &format!("{}/*.jpeg", self.path);

        let images: Vec<_> = glob(png).unwrap()
            .chain(glob(jpg).unwrap())
            .chain(glob(jpeg).unwrap())
            .into_iter().flatten().collect();
        
        let path = images.choose(&mut thread_rng());

        if path.is_none() {
            return None
        }

        let mut credit = None;
        let mut toml_path = None;
        let path = path.unwrap();

        let toml_str = format!("{}/{}.toml", self.path, path.file_stem().unwrap().to_string_lossy());
        let toml_buf = PathBuf::from(&toml_str);

        if toml_buf.exists() {
            toml_path = Some(toml_buf.clone());
            let value = fs::read_to_string(&toml_buf).unwrap(); // Shouldn't error because erm i check if it exists.
            let parsed_credit = toml::from_str::<CreditToml>(&value);

            if parsed_credit.is_ok() {
                credit = Some(parsed_credit.unwrap().credit);
            }
        }

        Some(
            Image {
                path: path.clone(),
                toml_path,
                credit
            }
        )
    }
}

impl Image {
    pub async fn make_into_embed(&self, agent: &BskyAgent) -> Option<Union<post::RecordEmbedRefs>> {
        let mut images = Vec::new();

        if let Ok(mut file) = File::open(&self.path).await {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.expect("read image file");
            let output = agent
                .api
                .com
                .atproto
                .repo
                .upload_blob(buf)
                .await
                .expect("upload blob");
        
                images.push(
                    images::ImageData {
                        alt: "".to_string(),
                        aspect_ratio: None,
                        image: output.data.blob,
                    }
                    .into(),
            )
        }
    
        return Some(Union::Refs(
            post::RecordEmbedRefs::AppBskyEmbedImagesMain(Box::new(
                images::MainData { images }.into(),
            )),
        ));
    }
}