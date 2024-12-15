use bsky_sdk::{api::{types::Union, app::bsky::embed::images, app::bsky::feed::post}, BskyAgent};
use glob::glob;
use std::{path::PathBuf, fs};
use tokio::{fs::File, io::AsyncReadExt};
use rand::{seq::SliceRandom, thread_rng};

#[derive(Clone, Debug)]
pub struct Images {
    path: String,
    uploaded_path: String,
}

#[derive(Clone, Debug)]
pub struct Image {
    path: PathBuf
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
            path: "./images".to_string(),
            uploaded_path: "./uploaded".to_string(),
        }
    }
}

impl Images {
    pub fn move_image(&self, image: &Image) {
        let path = &image.path;
        let file_name = path.file_name().unwrap();

        let dist = format!("{}/{}", self.uploaded_path, file_name.to_string_lossy());
        if let Err(e) = std::fs::rename(&path, &dist) {
            eprintln!("Failed to move file '{:?}' to '{:?}': '{}'", path, dist, e);
        }
    }

    pub fn get_random_image(&mut self) -> Option<Image> {
        let glob_path = format!("{}/*", self.path);
        let images: Vec<_> = glob(&glob_path)
            .expect(format!("Failed to glob file path '{}'", &glob_path).as_str())
            .into_iter().flatten().collect();
        
        let path = images.choose(&mut thread_rng());

        if path.is_none() {
            return None
        }

        Some(
            Image {
                path: path.unwrap().clone()
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
                        alt: "sata-andagi.moe".to_string(),
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