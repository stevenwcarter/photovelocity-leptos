#![allow(clippy::unnecessary_unwrap, clippy::needless_return)]
use crate::context::GraphQLContext;
use crate::folder::FolderSvc;
use crate::pgp::AuthName;
use crate::EndsWithAny;
use crate::Folder;
use crate::{base_folder, Image};
use async_recursion::async_recursion;
use cache_loader_async::backing::HashMapBacking;
use cache_loader_async::cache_api::{CacheEntry, LoadingCache};
use image::imageops::FilterType;
use image::{DynamicImage, ImageError as ImgError};
use log::*;
use rayon::prelude::*;
use std::{fmt, fs, path::Path};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use webp::{Encoder, WebPMemory};

pub struct ImageCache {
    pub cache: ImageCacheData,
}

impl Default for ImageCache {
    fn default() -> Self {
        let cache = LoadingCache::new(move |key: ImageCacheKey| async move {
            warn!("key path is {}", key.path);
            ImageSvc::list_internal(&key.path, &key.auth_type).await
        });

        Self { cache }
    }
}

impl Image {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Image {
    pub fn filename(&self) -> &str {
        self.path.as_str()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ImageCacheKey {
    path: String,
    auth_type: Option<AuthName>,
}

pub type ImageCacheBacking = HashMapBacking<ImageCacheKey, CacheEntry<Vec<Image>, ImageError>>;

pub type ImageCacheData = LoadingCache<ImageCacheKey, Vec<Image>, ImageError, ImageCacheBacking>;

#[derive(Debug, Clone)]
pub enum ImageError {
    NotAllowed,
    ThumbError,
    FsError,
    CacheError,
}
impl std::error::Error for ImageError {}
impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ImageError::NotAllowed => write!(f, "Operation not allowed"),
            ImageError::ThumbError => write!(f, "Could not generate thumbnail"),
            ImageError::FsError => write!(f, "Fs error"),
            ImageError::CacheError => write!(f, "Cache error"),
        }
    }
}

fn get_base_folder() -> String {
    let base_folder = base_folder();
    if base_folder.ends_with('/') {
        base_folder.strip_suffix('/').unwrap().to_string()
    } else {
        base_folder
    }
}
fn strip_slashes(filename: &str) -> &str {
    let filename = if filename.starts_with('/') {
        filename.strip_prefix('/').unwrap()
    } else {
        filename
    };

    let filename = if filename.ends_with('/') {
        filename.strip_suffix('/').unwrap()
    } else {
        filename
    };

    filename
}

#[derive(Clone)]
pub struct ImageSvc {}

impl ImageSvc {
    pub async fn is_not_hidden(path: &str, auth_type: &Option<AuthName>) -> bool {
        !ImageSvc::is_hidden(path, auth_type).await
    }

    pub async fn is_hidden(path: &str, auth_type: &Option<AuthName>) -> bool {
        trace!("Checking if hidden with auth type {:?}", auth_type);
        let folder = base_folder() + strip_slashes(path) + "/.hide";
        let file = Path::new(&folder);

        let hidden_exists = file.exists();

        // no hidden file, definitely not hidden
        if !hidden_exists {
            return false;
        }

        // hidden exists, and no auth passed, hidden
        if auth_type.is_none() {
            return true;
        }

        let auth_type = auth_type.as_ref().unwrap();

        if auth_type.name.eq("super") {
            return false;
        }

        if auth_type.name.is_empty() {
            return true;
        }

        let file_contents = tokio::fs::read_to_string(&folder).await.unwrap();

        !file_contents
            .split('\n')
            .any(|l| l.trim().eq(auth_type.name.trim()))
    }

    pub async fn list(context: &GraphQLContext, folder: &str) -> Result<Vec<Image>, ImageError> {
        context
            .image_cache
            .cache
            .get(ImageCacheKey {
                path: folder.to_string(),
                auth_type: context.auth.clone(),
            })
            .await
            .map_err(|_| ImageError::CacheError)
    }

    pub async fn generate_images_for_folder(
        context: &GraphQLContext,
        folder: &str,
        auth_type: &Option<AuthName>,
    ) -> Result<Vec<Image>, ImageError> {
        let images = context
            .image_cache
            .cache
            .get(ImageCacheKey {
                path: folder.to_string(),
                auth_type: auth_type.clone(),
            })
            .await
            .map_err(|_| ImageError::CacheError)?;

        let inner_images = images.clone();
        tokio::task::spawn_blocking(move || {
            if let Some(image) = inner_images.first() {
                let thumb_directory = Self::get_thumb_dirname(&image.path);
                let thumb_directory = Path::new(&thumb_directory);
                if !thumb_directory.exists() {
                    fs::create_dir(thumb_directory).unwrap();
                }
                inner_images.par_iter().for_each(|file| {
                    let res =
                        ImageSvc::generate_thumbnails(&file.path, vec![150, 300, 600, 1200, 2400]);
                    if res.is_err() {
                        println!("Could not generate thumbnails for {}", &file.path);
                    }
                });
            }
        });

        Ok(images)
    }

    async fn list_internal(
        folder: &str,
        auth_type: &Option<AuthName>,
    ) -> Result<Vec<Image>, ImageError> {
        let folder = folder.strip_prefix("/").unwrap();
        if folder.contains("..") {
            eprintln!("Attempt to traverse upward");
            return Err(ImageError::NotAllowed);
        }

        if Self::is_hidden(folder, auth_type).await {
            eprintln!("Attempt to view a hidden directory");
            return Err(ImageError::NotAllowed);
        }

        let paths_res = fs::read_dir(base_folder() + folder).map_err(|_| ImageError::FsError)?;
        let mut paths: Vec<Image> = paths_res
            .into_iter()
            .filter(|f| !f.as_ref().unwrap().metadata().unwrap().is_dir())
            .map(|p| p.unwrap().path().to_str().unwrap().to_string())
            .map(|p| p.replace(base_folder().as_str(), ""))
            .filter(|p| p.to_lowercase().as_str().ends_with_any(&[".jpg", ".jpeg"]))
            .map(Image::new)
            .collect();

        // TODO fix this slow copy
        paths.sort_by_key(|k| k.path.to_string());
        Ok(paths)
    }

    fn get_image_filename(filename: &str) -> String {
        let base_folder = get_base_folder();
        let filename = strip_slashes(filename);
        format!("{}/{}", base_folder, filename)
    }

    #[async_recursion]
    pub async fn get_folder_thumbnail(
        context: &GraphQLContext,
        folder: &str,
        size: u32,
    ) -> Result<Vec<u8>, ImageError> {
        let thumb_path = format!("{}/thumb", folder);
        let full_thumb_path = format!("{}{}-{}", base_folder(), thumb_path, size);
        let file_res = tokio::fs::File::open(full_thumb_path).await;
        if let Ok(mut file) = file_res {
            let mut contents: Vec<u8> = vec![];
            let read_res = file.read_to_end(&mut contents).await;
            if read_res.is_ok() {
                return Ok(contents);
            }
        }
        let thumb = ImageSvc::thumbnail(context, &thumb_path, size).await;
        if let Ok(thumb) = thumb {
            return Ok(thumb);
        }

        let files_res = Self::list_internal(folder, &context.auth).await;
        let files: Vec<Image> = files_res.unwrap_or_default();

        if !files.is_empty() {
            return ImageSvc::thumbnail(context, &files[0].path, size).await;
        }

        let folders_res = FolderSvc::list(context, folder).await;
        let folders: Vec<Folder> = folders_res.unwrap_or_default();

        if folders.is_empty() {
            return Err(ImageError::ThumbError);
        }

        let inner_folder = &folders[0];
        let image_data = Self::get_folder_thumbnail(context, &inner_folder.path, size).await?;

        info!("Creating thumb file {}", thumb_path);
        let thumb_filename = format!("{}{}-{}", base_folder(), thumb_path, size);
        let mut file = tokio::fs::File::create(thumb_filename)
            .await
            .expect("Could not create thumb file");
        file.write_all(&image_data)
            .await
            .expect("Could not write to thumb file");
        Ok(image_data)
    }

    fn get_thumb_dirname(filename: &str) -> String {
        let base_folder = get_base_folder();
        let pos = filename.rfind('/');
        let (folder, _) = if let Some(pos) = pos {
            filename.split_at(pos)
        } else {
            ("/", filename)
        };
        let folder = strip_slashes(folder);

        if folder.is_empty() {
            format!("{}/.thumbs", base_folder)
        } else {
            format!("{}/{}/.thumbs", base_folder, folder)
        }
    }
    fn get_thumb_filename(filename: &str, size: u32) -> String {
        let base_folder = get_base_folder();
        let pos = filename.rfind('/');
        let (folder, filename) = if let Some(pos) = pos {
            filename.split_at(pos)
        } else {
            ("/", filename)
        };
        let folder = strip_slashes(folder);
        let filename = strip_slashes(filename);

        if folder.is_empty() {
            format!("{}/.thumbs/{}-{}.webp", base_folder, filename, size)
        } else {
            format!(
                "{}/{}/.thumbs/{}-{}.webp",
                base_folder, folder, filename, size
            )
        }
    }

    pub async fn thumbnail(
        context: &GraphQLContext,
        filename: &str,
        size: u32,
    ) -> Result<Vec<u8>, ImageError> {
        let thumb_directory = Self::get_thumb_dirname(filename);
        let thumb_directory_path = Path::new(&thumb_directory);
        if !thumb_directory_path.exists() {
            trace!("Creating directory: {}", &thumb_directory);
            let result = fs::create_dir(thumb_directory_path);
            if result.is_err() {
                error!(
                    "Could not create thumb directory: {} {:?}",
                    thumb_directory,
                    result.err()
                );
            }
        }
        let thumb_filename = Self::get_thumb_filename(filename, size);
        let file = Path::new(&thumb_filename);
        if !file.exists() {
            Self::generate_thumbnail(filename, size).await?;
        }

        let pos = filename.rfind('/');
        let (folder, _) = if let Some(pos) = pos {
            filename.split_at(pos)
        } else {
            ("/", filename)
        };
        if Self::is_hidden(folder, &context.auth).await {
            error!(
                "Attempt to view a hidden directory: {folder} with auth {:?}",
                &context.auth
            );
            return Err(ImageError::NotAllowed);
        }

        tokio::fs::read(&thumb_filename)
            .await
            .map_err(|_| ImageError::FsError)
    }

    async fn generate_thumbnail(filename: &str, size: u32) -> Result<(), ImageError> {
        let thumb_filename = Self::get_thumb_filename(filename, size);
        let image_filename = Self::get_image_filename(filename);
        let img: Result<DynamicImage, ImgError> = image::open(image_filename);

        if img.is_err() {
            return Ok(());
        }

        let img = img.unwrap();
        let result = tokio::task::spawn_blocking(move || {
            let rgb = img.into_rgb8();
            let data: DynamicImage =
                DynamicImage::ImageRgb8(rgb).resize(size, size, FilterType::CatmullRom);
            let encoder: Encoder = Encoder::from_image(&data).unwrap();
            let webp: WebPMemory = encoder.encode(82f32);
            let result =
                std::fs::write(&thumb_filename, &*webp).map_err(|_| ImageError::ThumbError);
            if result.is_err() {
                println!("Could not create thumb: {}", &thumb_filename);
            }
        })
        .await;

        if result.is_err() {
            println!("Could not asynchronously run the generation");
        }

        Ok(())
    }
    fn generate_thumbnails(filename: &str, sizes: Vec<u32>) -> Result<(), ImageError> {
        let image_filename = Self::get_image_filename(filename);
        let thumb_filename = Self::get_thumb_filename(filename, 2400);
        let file = Path::new(&thumb_filename);
        if !file.exists() {
            let img: Result<DynamicImage, ImgError> = image::open(image_filename);
            let img = img.unwrap();
            let rgb = img.into_rgb8();

            sizes.par_iter().for_each(|size| {
                let thumb_filename = Self::get_thumb_filename(filename, *size);
                let file = Path::new(&thumb_filename);
                if !file.exists() {
                    let data: DynamicImage = DynamicImage::ImageRgb8(rgb.clone()).resize(
                        *size,
                        *size,
                        FilterType::CatmullRom,
                    );
                    let encoder: Encoder = Encoder::from_image(&data).unwrap();
                    let webp: WebPMemory = encoder.encode(82f32);
                    let result =
                        std::fs::write(&thumb_filename, &*webp).map_err(|_| ImageError::ThumbError);

                    if result.is_err() {
                        println!(
                            "Could not save thumb: {} {:?}",
                            &thumb_filename,
                            result.err()
                        );
                    }
                }
            });
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    pub async fn it_reads_directories() {
        dotenvy::from_filename(".env.test").ok();
        let result = ImageSvc::list_internal("/", &None).await.unwrap();
        assert_eq!(result.len(), 20);
    }
    #[tokio::test]
    pub async fn it_fails_on_missing_directories() {
        dotenvy::from_filename(".env.test").ok();
        let result = ImageSvc::list_internal(
            "/testfoobar",
            &Some(AuthName {
                name: "super".to_string(),
            }),
        )
        .await;
        assert!(result.is_err());
    }
    #[tokio::test]
    pub async fn is_hidden_no_auth_branch() {
        dotenvy::from_filename(".env.test").ok();
        let result = ImageSvc::is_hidden("/test", &None).await;
        assert!(result);
    }
    #[tokio::test]
    pub async fn is_hidden_super_auth_branch() {
        dotenvy::from_filename(".env.test").ok();
        let result = ImageSvc::is_hidden(
            "/test",
            &Some(AuthName {
                name: "super".to_string(),
            }),
        )
        .await;
        assert!(!result);
    }
    #[tokio::test]
    pub async fn is_hidden_empty_auth_branch() {
        dotenvy::from_filename(".env.test").ok();
        let result = ImageSvc::is_hidden(
            "/test",
            &Some(AuthName {
                name: "".to_string(),
            }),
        )
        .await;
        assert!(result);
    }
    #[tokio::test]
    pub async fn is_hidden_named_auth_branch() {
        dotenvy::from_filename(".env.test").ok();
        let result = ImageSvc::is_hidden(
            "/test",
            &Some(AuthName {
                name: "test".to_string(),
            }),
        )
        .await;
        assert!(!result);
    }
    #[tokio::test]
    pub async fn cached_and_uncached_match() {
        let context = GraphQLContext::default();
        dotenvy::from_filename(".env.test").ok();
        // call internal first
        let uncached = ImageSvc::list_internal("/", &None).await.unwrap();
        let cached = ImageSvc::list(&context, "/").await.unwrap();
        assert_eq!(uncached[0].path, cached[0].path);
    }
    #[tokio::test]
    pub async fn it_generates_a_thumbnail() {
        dotenvy::from_filename(".env.test").ok();
        let result = ImageSvc::generate_thumbnail("/SidawayFamilyShoot11574.jpg", 222).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    pub async fn it_generates_and_returns_a_thumbnail() {
        dotenvy::from_filename(".env.test").ok();
        let path = "/SidawayFamilyShoot11574.jpg";
        let size = 222;
        let result = ImageSvc::generate_thumbnail(path, size).await;
        assert!(result.is_ok());
        let thumb = ImageSvc::thumbnail(&GraphQLContext::default(), path, size).await;
        assert!(thumb.is_ok());
        let thumb = thumb.unwrap();
        assert!(thumb.len() > 100);
    }
    #[test]
    pub fn it_gets_thumb_filename_for_root() {
        dotenvy::from_filename(".env.test").ok();
        let filename = ImageSvc::get_thumb_filename("/test.jpg", 222);
        assert_eq!(filename, "./photos/.thumbs/test.jpg-222.webp");
    }
    #[test]
    pub fn it_gets_thumb_filename_for_nested() {
        dotenvy::from_filename(".env.test").ok();
        let filename = ImageSvc::get_thumb_filename("/Pets/D75_0360.jpg", 222);
        assert_eq!(filename, "./photos/Pets/.thumbs/D75_0360.jpg-222.webp");
    }
    #[test]
    pub fn it_strips_slashes() {
        let result = strip_slashes("/asdf.jpg");
        assert_eq!(result, "asdf.jpg");
    }
    #[test]
    pub fn it_leaves_non_slashed_alone() {
        let result = strip_slashes("asdf.jpg");
        assert_eq!(result, "asdf.jpg");
    }
    #[test]
    pub fn it_strips_trailing_slashes() {
        let result = strip_slashes("asdf.jpg/");
        assert_eq!(result, "asdf.jpg");
    }
}
