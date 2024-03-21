use std::sync::Arc;

use crate::{folder::FolderCache, image::ImageCache, pgp::AuthName};

#[derive(Clone)]
pub struct GraphQLContext {
    pub folder_cache: Arc<FolderCache>,
    pub image_cache: Arc<ImageCache>,
    pub auth: Option<AuthName>,
}

impl Default for GraphQLContext {
    fn default() -> Self {
        let folder_cache = Arc::new(FolderCache::default());
        let image_cache = Arc::new(ImageCache::default());

        Self {
            folder_cache,
            image_cache,
            auth: None,
        }
    }
}

impl GraphQLContext {
    pub fn attach_session(&self, auth: Option<AuthName>) -> Arc<Self> {
        Arc::new(Self {
            auth: auth.clone(),
            folder_cache: self.folder_cache.clone(),
            image_cache: self.image_cache.clone(),
        })
    }
}
