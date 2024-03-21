use crate::{app::folder_thumb::FolderThumb, error_template::ErrorTemplate, Folder};
use leptos::*;
use leptos_router::*;

#[component]
pub fn FolderView() -> impl IntoView {
    let location = Signal::derive(|| use_location());
    let folders = create_resource(
        move || location().pathname,
        move |pathname| get_folders(pathname.get()),
    );

    view! {
        <div class="flex overflow-x-scroll p-12 scrollbar scrollbar-track-[#232323] scrollbar-thumb-[#535353]">
            <Transition>
                <FolderList folders=folders />
            </Transition>
        </div>
    }
}

#[component]
pub fn FolderList(
    folders: Resource<Memo<String>, Result<Vec<Folder>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors /> }
            }>

                {move || {
                    folders
                        .get()
                        .map(move |folders| match folders {
                            Err(e) => {
                                view! { <pre class="error">"Server error: " {e.to_string()}</pre> }
                                    .into_view()
                            }
                            Ok(folders) => {
                                folders
                                    .into_iter()
                                    .map(move |folder| {
                                        let (folder_path, _) = create_signal(folder.path);
                                        view! { <FolderThumb folder_path=folder_path /> }
                                    })
                                    .collect_view()
                            }
                        })
                        .unwrap_or_default()
                }}

            </ErrorBoundary>
        </Transition>
    }
}

#[server]
pub async fn get_folders(pathname: String) -> Result<Vec<Folder>, ServerFnError> {
    use crate::api::SessionContext;
    use crate::folder::FolderSvc;
    use leptos_axum::extract;
    use log::*;

    let SessionContext(context): SessionContext = extract().await?;

    info!("Pathname is: {pathname}");

    FolderSvc::list(&context, &pathname)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
