use crate::context::GraphQLContext;
use crate::pgp::AuthName;
use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::RequestPartsExt;
use axum::{async_trait, http::StatusCode, Extension, Json, Router};
use axum_extra::extract::CookieJar;
use log::*;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;
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

    trace!("Auth for folder thumb is: {:?}", context.auth);
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

#[derive(Deserialize, Debug)]
struct QueryParameters {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    auth: Option<String>,
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
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

        let Query(query) = parts
            .extract::<Query<QueryParameters>>()
            .await
            .expect("could not process query parameters");

        let Extension(context) = parts
            .extract::<Extension<Arc<GraphQLContext>>>()
            .await
            .map_err(|err| {
                error!("Error retrieving extension: {:?}", err);
                err.into_response()
            })?;

        let session_id = match (cookie_jar.get("X-Login"), query.auth) {
            (None, Some(auth)) => Some(auth),
            (Some(cookie), _) => Some(cookie.value().to_string()),
            _ => None,
        };

        let Some(session_id) = session_id else {
            return Ok(Self(context.clone()));
        };

        let auth_type = AuthName::parse(session_id);

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
