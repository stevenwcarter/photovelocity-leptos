use crate::context::GraphQLContext;
use crate::pgp::AuthName;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::RequestPartsExt;
use axum::{async_trait, http::StatusCode, Extension, Json, Router};
use axum_extra::extract::CookieJar;
use log::*;
use serde::Serialize;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use axum::extract::Path;

pub mod login;

pub fn middleware() -> tower::ServiceBuilder<
    tower::layer::util::Stack<
        tower_http::compression::CompressionLayer,
        tower::layer::util::Identity,
    >,
> {
    ServiceBuilder::new().compression()
}

pub fn api_routes(context: Arc<GraphQLContext>) -> Router {
    use self::login::login_routes;
    // use self::voting::voting_routes;
    Router::new()
        .route("/test", get(get_test))
        .route("/folderThumb/:size/:folder", get(folder_thumbnail))
        .route("/imageThumb/:size/:image", get(image_thumbnail))
        // .nest("/vote", voting_routes(context.clone()))
        .nest("/login", login_routes(context.clone()))
        .layer(Extension(context.clone()))
}

pub async fn folder_thumbnail(
    Path((size, folder)): Path<(u32, String)>,
    SessionContext(context): SessionContext,
) -> Response {
    use http::header;

    use crate::image::ImageSvc;

    info!("Auth for folder thumb is: {:?}", context.auth);
    let result = ImageSvc::get_folder_thumbnail(&context, &folder, size).await;
    match result {
        Err(e) => {
            error!("Error retrieving thumbnail: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
        Ok(data) => (
            StatusCode::OK,
            axum::response::AppendHeaders([(header::CONTENT_TYPE, "image/webp")]),
            data,
        )
            .into_response(),
    }
}
pub async fn image_thumbnail(
    Path((size, image)): Path<(u32, String)>,
    SessionContext(context): SessionContext,
) -> Response {
    use http::header;

    use crate::image::ImageSvc;

    let result = ImageSvc::thumbnail(&context, &image, size).await;
    match result {
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        Ok(data) => (
            StatusCode::OK,
            axum::response::AppendHeaders([(header::CONTENT_TYPE, "image/webp")]),
            data,
        )
            .into_response(),
    }
}

pub async fn get_test() -> &'static str {
    " hello world"
}

pub fn err_wrapper<T: Serialize>(result: anyhow::Result<T>) -> impl IntoResponse {
    Json(
        result
            .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))
            .unwrap(),
    )
}

pub struct SessionContext(pub Arc<GraphQLContext>);

#[async_trait]
impl<S> FromRequestParts<S> for SessionContext
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let cookie_jar = parts.extract::<CookieJar>().await.unwrap();

        let Extension(context) = parts
            .extract::<Extension<Arc<GraphQLContext>>>()
            .await
            .map_err(|err| {
                error!("Error retrieving extension: {:?}", err);
                err.into_response()
            })?;

        let Some(session_id) = cookie_jar.get("X-Login") else {
            return Ok(Self(context.clone()));
        };
        let login_cookie = session_id.value();
        let auth_type = AuthName::parse(login_cookie);

        let context = context.attach_session(auth_type);

        Ok(Self(context))
    }
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {:?}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
