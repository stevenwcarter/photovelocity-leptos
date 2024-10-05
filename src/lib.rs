#![allow(clippy::unnecessary_unwrap, clippy::needless_return)]
pub mod app;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;

use std::iter::Extend;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

#[cfg(feature = "ssr")]
use crate::pgp::{read_gpg, AuthName};
#[cfg(feature = "ssr")]
use std::env;
#[cfg(feature = "ssr")]
use std::fmt;
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[cfg(feature = "ssr")]
pub mod api;
#[cfg(feature = "ssr")]
pub mod context;
#[cfg(feature = "ssr")]
pub mod folder;
// #[cfg(feature = "ssr")]
// pub mod graphql;
#[cfg(feature = "ssr")]
pub mod hash;
#[cfg(feature = "ssr")]
pub mod image;
#[cfg(feature = "ssr")]
pub mod pgp;

thread_local! {
    static PATH_REGEX: Regex = Regex::new(r":(\d*):.*").expect("Could not compile regex");
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Folder {
    pub path: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub path: String,
}

/// Return an environment variable typed generically
///
/// ```
/// use photo365::get_env;
/// assert!(get_env("PATH", "test").len() > 4);
/// ````
#[cfg(feature = "ssr")]
pub fn get_env(search_key: &str, default: &str) -> String {
    if let Some(value) = env::vars()
        .filter(|(key, _)| key.eq(search_key))
        .map(|(_, value)| value)
        .next()
    {
        value
    } else {
        default.to_string()
    }
}

/// Return an environment variable typed generically
///
/// ```
/// use photo365::get_env_typed;
/// assert!(get_env_typed::<u16>("SHLVL", 9) > 0);
/// ````
#[cfg(feature = "ssr")]
pub fn get_env_typed_result<T>(search_key: &str) -> anyhow::Result<T>
where
    T: FromStr + fmt::Debug,
{
    use anyhow::bail;

    if let Some(value) = env::vars()
        .filter(|(key, _)| key.eq(search_key))
        .map(|(_, value)| value)
        .next()
    {
        let result = value.parse::<T>();
        match result {
            Ok(res) => Ok(res),
            Err(_) => bail!("Could not parse '{value}' for '{search_key}",),
        }
    } else {
        bail!("env variable for '{search_key}' not found");
    }
}

/// Return an environment variable typed generically
///
/// ```
/// use photo365::get_env_typed;
/// assert!(get_env_typed::<u16>("SHLVL", 9) > 0);
/// ````
#[cfg(feature = "ssr")]
pub fn get_env_typed<T>(search_key: &str, default: T) -> T
where
    T: FromStr + fmt::Debug,
{
    if let Some(value) = env::vars()
        .filter(|(key, _)| key.eq(search_key))
        .map(|(_, value)| value)
        .next()
    {
        let value = value.parse::<T>();
        match value {
            Ok(value) => value,
            Err(_) => default,
        }
    } else {
        default
    }
}

#[cfg(feature = "ssr")]
fn base_folder() -> String {
    get_env("PHOTO_DIR", "/photos/")
}

#[cfg(feature = "ssr")]
pub fn read_pgp_auth_type(auth: Option<String>) -> Option<AuthName> {
    let auth = auth?;
    let msg = urlencoding::decode(&auth).ok()?;
    read_gpg(msg.as_ref())
}

trait Interleave<T> {
    fn interleave(&self, portion: &dyn Fn() -> T) -> Vec<T>;
}

impl<T: Clone> Interleave<T> for Vec<T> {
    fn interleave(&self, portion: &dyn Fn() -> T) -> Vec<T> {
        self.iter()
            .flat_map(|item| vec![item.clone(), portion().clone()])
            .take(self.len() * 2 - 1)
            .collect()
    }
}

#[cfg(feature = "ssr")]
trait EndsWithAny {
    fn ends_with_any(&self, suffixes: &[&str]) -> bool;
}

#[cfg(feature = "ssr")]
impl EndsWithAny for &String {
    fn ends_with_any(&self, suffixes: &[&str]) -> bool {
        for s in suffixes {
            if self.ends_with(s) {
                return true;
            }
        }

        false
    }
}
#[cfg(feature = "ssr")]
impl EndsWithAny for &str {
    fn ends_with_any(&self, suffixes: &[&str]) -> bool {
        for s in suffixes {
            if self.ends_with(s) {
                return true;
            }
        }

        false
    }
}

pub fn nth_index_of(s: &str, c: char, n: usize) -> Option<usize> {
    let chars = s.chars();
    let mut pos = 0;
    let mut count = 0;

    for ch in chars {
        pos += 1;
        if ch == c {
            count += 1;
            if count == n {
                return Some(pos);
            }
        }
    }

    None
}

pub fn get_url_parts(url: &str) -> Vec<String> {
    let mut parts = vec!["/".to_string()];

    let url = url.strip_prefix("/").unwrap_or_default();
    let url = PATH_REGEX.with(|a| a.replace_all(url, ""));

    let add_parts: Vec<String> = url
        .split('/')
        .filter(|a| !a.is_empty())
        .map(|a| a.to_string())
        .collect();

    parts.extend(add_parts);

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "ssr")]
    fn it_gets_env_variabls() {
        assert!(get_env("PATH", "test").len() > 4);
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn it_gets_env_variabls_default() {
        assert!(get_env("FOOBAR", "test").eq("test"));
    }

    #[ignore]
    #[test]
    #[cfg(feature = "ssr")]
    fn it_gets_typed_env_variables() {
        assert!(get_env_typed::<u16>("SHLVL", 9) > 0);
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn it_gets_typed_env_variables_default() {
        assert!(get_env_typed::<u16>("FOOBAR", 9) == 9);
    }

    #[test]
    fn it_gets_url_parts() {
        assert_eq!(
            get_url_parts("/a/b/c"),
            vec![
                "/".to_string(),
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ]
        );
    }

    #[test]
    fn it_interleaves_numbers_properly() {
        let a: Vec<u32> = vec![1, 2, 3];

        assert_eq!(a.interleave(&|| 0), vec![1, 0, 2, 0, 3]);
    }
    #[test]
    fn it_interleaves_strs() {
        let a = vec!["a", "b", "c"];

        assert_eq!(a.interleave(&|| "."), vec!["a", ".", "b", ".", "c",]);
    }
}
