use breadcrumb_nav::BreadcrumbNav;
use codee::string::FromToStringCodec;
use folder_list::*;
use image_list::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::*;

mod breadcrumb_nav;
mod folder_list;
mod folder_thumb;
mod image_list;
mod image_thumb;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/photo-365.css" />

        // sets the document title
        <Title text="PhotoVelocity" />

        <Body class="p-4 text-white bg-[#232323]" />

        // content for this welcome page
        <Router fallback=|| {
            view! {
                // let mut outside_errors = Errors::default();
                // outside_errors.insert_with_default_key(AppError::NotFound);
                <HomePage />
            }
                .into_view()
        }>
            <main>
                <Routes>
                    <Route path="/" view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Debug, Params, PartialEq)]
struct AuthParams {
    auth: Option<String>,
}
/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let (_auth, set_auth) = use_cookie::<String, FromToStringCodec>("X-Login");
    let params = use_query::<AuthParams>();
    let auth_val = move || {
        params.with(|params| match params {
            Ok(params) => {
                logging::log!("Params: {:?}", params.auth);
                params.auth.clone()
            }
            _ => None,
        })
    };
    create_isomorphic_effect(move |_| {
        logging::log!("Auth {:?}", auth_val());
        let auth = auth_val();
        if auth.is_some() {
            set_auth(auth);
        }
    });
    // Creates a reactive value to update the button
    let location = use_location();
    let images = create_resource(
        move || location.pathname,
        move |pathname| get_images(pathname.get()),
    );

    view! {
        <a class="text-right no-underline">
            <h1 class="text-4xl font-semibold">"PhotoVelocity"</h1>
        </a>
        <BreadcrumbNav hide_url=false />
        <FolderView />
        <div class="flex flex-col flex-wrap">
            <Transition>
                <ImageList images=images />
            </Transition>
        </div>
    }
}
