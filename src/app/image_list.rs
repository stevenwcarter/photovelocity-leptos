use crate::{app::image_thumb::*, error_template::ErrorTemplate, Image};
use leptos::*;
use leptos_router::*;

#[component]
fn ImageView() -> impl IntoView {
    // Creates a reactive value to update the button
    let location = Signal::derive(|| use_location());
    let images = create_resource(
        move || location().pathname,
        move |pathname| get_images(pathname.get()),
    );

    view! {
        <div class="flex flex-col flex-wrap">
            <Transition>
                <ImageList images=images />
            </Transition>
        </div>
    }
}
#[component]
fn TextView() -> impl IntoView {
    // Creates a reactive value to update the button
    let location = Signal::derive(|| use_location());
    let text = create_resource(
        move || location().pathname,
        move |pathname| get_text(pathname.get()),
    );

    view! {
        <Transition fallback=move || view! { "" }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors /> }
            }>
                {move || {
                    match text.get() {
                        Some(Ok(text_val)) => view! { <p class="py-4">{text_val}</p> }.into_view(),
                        _ => view! { "" }.into_view(),
                    }
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn ImageList(
    images: Resource<Memo<String>, Result<Vec<Image>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap">
            <TextView />
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors /> }
                }>

                    {move || {
                        let images = images
                            .get()
                            .map(move |images| match images {
                                Err(e) => {
                                    view! {
                                        <pre class="error">"Server error: " {e.to_string()}</pre>
                                    }
                                        .into_view()
                                }
                                Ok(images) => {
                                    images
                                        .into_iter()
                                        .map(move |image| {
                                            view! { <ImageThumb image_path=image.path /> }
                                        })
                                        .collect_view()
                                }
                            })
                            .unwrap_or_default();
                        view! { <div class="flex flex-wrap">{images}</div> }
                    }}

                </ErrorBoundary>
            </Transition>
        </div>
    }
}

#[server]
pub async fn get_text(pathname: String) -> Result<String, ServerFnError> {
    use crate::api::SessionContext;
    use crate::folder::FolderSvc;
    use leptos_axum::extract;
    use log::*;

    let SessionContext(context): SessionContext = extract().await?;

    info!("Text Pathname is: {pathname}");

    Ok(FolderSvc::get_folder_text(&pathname, &context.auth)
        .await
        .unwrap_or_default())
}

#[server]
pub async fn get_images(pathname: String) -> Result<Vec<Image>, ServerFnError> {
    use crate::api::SessionContext;
    use crate::image::ImageSvc;
    use leptos_axum::extract;
    use log::*;

    let SessionContext(context): SessionContext = extract().await?;

    info!("Pathname is: {pathname}");

    ImageSvc::list(&context, &pathname)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
