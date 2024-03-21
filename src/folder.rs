use crate::context::GraphQLContext;
use crate::pgp::AuthName;
use crate::Folder;
use crate::{base_folder, image::ImageSvc};
use cache_loader_async::backing::HashMapBacking;
use cache_loader_async::cache_api::{CacheEntry, LoadingCache};
use futures::stream::{self, StreamExt};
use log::*;
use std::path::Path;
use std::{fmt, fs};
use tokio::fs::read_to_string;

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct FolderCacheKey {
    path: String,
    auth_type: Option<AuthName>,
}

pub type FolderCacheBacking = HashMapBacking<FolderCacheKey, CacheEntry<Vec<Folder>, FolderError>>;

pub type FolderCacheData =
    LoadingCache<FolderCacheKey, Vec<Folder>, FolderError, FolderCacheBacking>;

pub struct FolderCache {
    pub cache: FolderCacheData,
}

impl Default for FolderCache {
    fn default() -> Self {
        let cache = LoadingCache::new(move |key: FolderCacheKey| async move {
            // println!("folder Cache miss for {}", key.path);
            FolderSvc::list_internal(&key.path, key.auth_type).await
        });

        Self { cache }
    }
}

impl Folder {
    pub fn new(path: String, text: Option<String>) -> Self {
        Self { path, text }
    }
}

impl Folder {
    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

#[derive(Debug, Clone)]
pub enum FolderError {
    CacheError,
    NotAllowed,
    FsError,
}
impl std::error::Error for FolderError {}
impl fmt::Display for FolderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FolderError::NotAllowed => write!(f, "Operation not allowed"),
            FolderError::FsError => write!(f, "Fs error"),
            FolderError::CacheError => write!(f, "Cache error"),
        }
    }
}

pub struct FolderSvc {}

impl FolderSvc {
    pub async fn get_folder_text(folder: &str, auth_type: &Option<AuthName>) -> Option<String> {
        let folder_path = format!("{}{}", base_folder(), folder);
        let not_hidden = ImageSvc::is_not_hidden(&folder_path, auth_type).await;
        if not_hidden {
            let index_path = format!("{}{}/index.txt", base_folder(), folder);
            // println!("Text path: {}", index_path);

            let index_path = index_path.replace("//", "/");
            // println!("Text path after replace: {}", index_path);
            let index_path = Path::new(index_path.as_str());
            let res = read_to_string(index_path).await;
            res.ok()
        } else {
            None
        }
    }
    pub async fn list(context: &GraphQLContext, folder: &str) -> Result<Vec<Folder>, FolderError> {
        info!("Folder: {folder}");
        context
            .folder_cache
            .cache
            .get(FolderCacheKey {
                path: folder.to_string(),
                auth_type: context.auth.clone(),
            })
            .await
            .map_err(|e| {
                println!("Error loading from cache: {:#?}", e);
                FolderError::CacheError
            })
    }

    async fn list_internal(
        folder: &str,
        auth_type: Option<AuthName>,
    ) -> Result<Vec<Folder>, FolderError> {
        info!("Checking folders for {folder}");
        if folder.contains("..") {
            println!("Folder cannot contain '..'");
            return Err(FolderError::NotAllowed);
        }

        let paths_res = fs::read_dir(base_folder() + folder).map_err(|_| FolderError::FsError)?;
        Ok(stream::iter(
            paths_res
                .into_iter()
                .filter(|f| f.as_ref().is_ok())
                .filter(|f| f.as_ref().unwrap().metadata().is_ok())
                .filter(|f| f.as_ref().unwrap().metadata().unwrap().is_dir())
                .map(|p| p.unwrap().path().to_str().unwrap().to_string())
                .map(|p| p.replace(base_folder().as_str(), ""))
                .filter(|p| !p.contains(".thumbs")),
        )
        .filter_map(|p| async {
            let not_hidden = ImageSvc::is_not_hidden(p.as_str(), &auth_type).await;
            if not_hidden {
                let text: Option<String> = FolderSvc::get_folder_text(&p, &auth_type).await;
                Some(Folder::new(p, text))
            } else {
                None
            }
        })
        // .map(Folder::new)
        .collect::<Vec<Folder>>()
        .await)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(not(tarpaulin))]
    pub async fn it_reads_directories() {
        dotenvy::from_filename(".env.test").ok();
        let context = GraphQLContext::default();
        let result = FolderSvc::list(&context, "/").await.unwrap();
        assert_eq!(result.len(), 1);
        println!("{}", result[0].path);
        assert_eq!(result[0].path, "/Pets");
    }

    #[tokio::test]
    pub async fn it_reads_hidden_directories() {
        dotenvy::from_filename(".env.test").ok();
        let mut context = GraphQLContext::default();
        context.auth = Some(AuthName::new("super".to_string()));
        let result = FolderSvc::list(&context, "/").await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    pub async fn it_blocks_hidden_directories() {
        dotenvy::from_filename(".env.test").ok();
        let context = GraphQLContext::default();
        let result = FolderSvc::list(&context, "/test").await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    pub async fn it_blocks_backtracking_directories() {
        dotenvy::from_filename(".env.test").ok();
        let context = GraphQLContext::default();
        let result = FolderSvc::list(&context, "/../test").await;
        assert!(result.is_err());
    }
}
