use leptos::*;
use leptos_router::use_location;
use urlencoding::encode;

#[component]
pub fn FolderThumb(folder_path: ReadSignal<String>) -> impl IntoView {
    let encoded = move || {
        let encoded = folder_path();
        let encoded = encoded.as_str();
        let encoded = encode(encoded);
        encoded.to_string()
    };
    let img_path_x1 = move || format!("/api/v1/folderThumb/150/{}", encoded());
    let img_path_x2 = move || format!("/api/v1/folderThumb/300/{}", encoded());
    let path = move || use_location().pathname;
    let pretty_path = move || {
        let stripped = folder_path();
        let path = path().get();
        let stripped = stripped
            .strip_prefix(path.as_str())
            .unwrap_or_else(|| &stripped);
        let stripped = stripped.strip_prefix("/").unwrap_or(stripped);
        stripped.replace("-", " ")
    };

    view! {
        <div class="box-content flex relative flex-col p-6 -ml-32 bg-scroll bg-clip-border rounded-2xl shadow-2xl transition-all duration-200 first:ml-0 bg-origin-padding group peer peer-hover:translate-x-[130px] h-[200px] w-[225px] min-w-[225px] shadow-black/50 bg-[#676767] hover:translate-y-[-1rem]">
            <a href=folder_path>
                <header class="my-1 mx-1 ml-auto card-header">
                    <h2 class="mx-auto mt-1 text-2xl font-bold text-left text-transparent underline bg-clip-text bg-gradient-to-r shadow-none cursor-pointer decoration-solid decoration-[#343840] text-shadow-none from-[#ccc] to-[#66d] group-hover:from-[#eee] group-hover:to-[#55d]">
                        {pretty_path}
                    </h2>
                </header>
                <div class="overflow-hidden relative mx-auto w-full rounded-lg">
                    <img
                        src=img_path_x1.to_owned()
                        srcset=move || format!("{} 1x, {} 2x", img_path_x1(), img_path_x2())
                    />
                </div>
            </a>
        </div>
    }
}
