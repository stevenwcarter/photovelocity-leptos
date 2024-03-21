use crate::{nth_index_of, Interleave};
use icondata as i;
use leptos::*;
use leptos_icons::*;
use leptos_router::*;
use regex::Regex;

use crate::get_url_parts;

thread_local! {
    static PATH_PREFIX_REGEX: Regex = Regex::new(r"^/?").expect("Could not compile regex");
}

#[component]
pub fn BreadcrumbNav(hide_url: bool) -> impl IntoView {
    let location = Signal::derive(|| use_location());
    let url_parts = move || get_url_parts(&location().pathname.get());

    if hide_url {
        return view! {}.into_view();
    }

    let crumbs = move || {
        let parts = url_parts();
        let inner_parts = parts.clone();
        parts
        .into_iter()
        .enumerate()
        .map(move |(i, p)| {
            let name = match p.as_str() {
                "/" => "Home".to_string(),
                _ => p.replace('-', " ")
            };

            let p = (0..i+1).map(|i| inner_parts[i].clone()).collect::<Vec<String>>().join("/").replace("//", "/");
            let index = nth_index_of(p.as_str(), '/', i + 1).unwrap_or(p.len());

            let p = p[0..index].to_owned();
            logging::log!("=== NEW === Path is: {}", p);

            view! {
                <a
                    href=p
                    class="inline-block py-1 px-2 text-white bg-gray-500 rounded-full border border-gray-200 transition-transform ease-in-out transform hover:bg-gray-900 hover:border-gray-400 hover:scale-105 hover:-translate-y-0.5"
                >
                    {name}
                </a>
            }.into_view()
        })
        .collect::<Vec<View>>()
    };

    let interleave = move || view! { <Icon icon=i::FaArrowRightLongSolid /> }.into_view();
    // let crumbs_view = move || crumbs().interleave(&interleave);

    view! { <div class="flex gap-2 items-center">{move || crumbs().interleave(&interleave)}</div> }
        .into_view()
}
// #[component]
// pub fn BreadcrumbNav(hide_url: bool) -> impl IntoView {
//     let location = use_location();
//     let url_parts = move || get_url_parts(&location.pathname.get());

//     if hide_url {
//         return view! {}.into_view();
//     }

//     let crumbs = move || {
//         let parts = url_parts();
//         let parts = parts
//         .into_iter()
//         .map(|p| {
//             let name = match p.as_str() {
//                 "/" => "Home".to_string(),
//                 _ => p.clone(),
//             };

//             view! {
//                 <a
//                     href=p
//                     class="inline-block py-1 px-2 text-white bg-gray-500 rounded-full border border-gray-200 transition-transform ease-in-out transform hover:bg-gray-900 hover:border-gray-400 hover:scale-105 hover:-translate-y-0.5"
//                 >
//                     {name}
//                 </a>
//             }.into_view().into()
//         })
//         .collect::<Vec<Fragment>>();

//         parts
//             .iter()
//             .flat_map(|item| {
//                 let separator: Fragment = view! { <Icon icon=i::FaArrowRightLongSolid /> }
//                     .into_view()
//                     .into();
//                 vec![item.clone(), separator.clone()]
//             })
//             .take(parts.len() * 2 - 1)
//             .collect::<Vec<Fragment>>()
//     };

//     view! { <div class="flex gap-2 items-center">{crumbs}</div> }.into_view()
// }
