use axum::{Extension, Router};
use std::sync::Arc;

use super::middleware;
use crate::context::GraphQLContext;
#[allow(unused_imports)]
use log::*;

#[cfg(feature = "ssr")]
pub fn login_routes(context: Arc<GraphQLContext>) -> Router {
    Router::new()
        // .route("/", post(login_user))
        // .route("/sign-up", post(signup_user))
        // .route("/ping", get(ping_user))
        .layer(Extension(context.clone()))
        .layer(middleware())
}

// async fn signup_user(
//     Extension(context): Extension<Arc<GraphQLContext>>,
//     cookie_jar: CookieJar,
//     Json(user): Json<User>,
// ) -> Response {
//     let password_hash = &user.password_hash;

//     let password_hash = UserData::generate_password_hash(password_hash).unwrap();

//     let user = User {
//         password_hash,
//         ..user
//     };

//     let create_user_result = UserData::create_user(&context, &user);
//     if create_user_result.is_err() {
//         return (StatusCode::INTERNAL_SERVER_ERROR, "could not create user").into_response();
//     }

//     let session = Session::new(&context, &user);
//     let session_cookie = Cookie::build(("X-Login-Session-ID", session.uuid.clone()))
//         .path("/")
//         .secure(true)
//         .max_age(Duration::days(14))
//         .build();

//     (
//         StatusCode::CREATED,
//         cookie_jar.add(session_cookie),
//         Json(session),
//     )
//         .into_response()
// }

// async fn ping_user(
//     cookie_jar: CookieJar,
//     SessionContext(context): SessionContext,
// ) -> impl IntoResponse {
//     // TODO - update session expiration internally too
//     let session = context.session.clone().unwrap();
//     let session_cookie = Cookie::build(("X-Login-Session-ID", session.uuid.clone()))
//         .path("/")
//         .max_age(Duration::days(14))
//         .secure(true)
//         .build();

//     (
//         StatusCode::OK,
//         cookie_jar.add(session_cookie),
//         Json(session),
//     )
// }
// async fn login_user(
//     Extension(context): Extension<Arc<GraphQLContext>>,
//     cookie_jar: CookieJar,
//     Path(email): Path<String>,
//     password: String,
// ) -> Response {
//     let user_result = UserData::get_user_by_email(&context, &email);
//     if user_result.is_err() {
//         return (StatusCode::UNAUTHORIZED, "no user found").into_response();
//     }
//     let user_result = user_result.unwrap();
//     let password_hash = &user_result.password_hash;

//     let valid_password = UserData::validate_password(&password, password_hash);

//     if !valid_password {
//         return (StatusCode::UNAUTHORIZED, "wrong password").into_response();
//     }

//     let session = Session::new(&context, &user_result);

//     let session_cookie = Cookie::build(("X-Login-Session-ID", session.uuid.clone()))
//         .path("/")
//         .secure(true)
//         .max_age(Duration::days(14))
//         .build();

//     (
//         StatusCode::CREATED,
//         cookie_jar.add(session_cookie),
//         Json(session),
//     )
//         .into_response()
// }
