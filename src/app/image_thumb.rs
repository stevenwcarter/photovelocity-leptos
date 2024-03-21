use leptos::*;
use urlencoding::encode;

#[component]
pub fn ImageThumb(image_path: String) -> impl IntoView {
    let encoded = encode(&image_path);
    let img_path_x1 = format!("/api/v1/imageThumb/300/{encoded}",);
    let img_path_x2 = format!("/api/v1/imageThumb/600/{encoded}",);

    view! {
        <div class="flex overflow-hidden relative justify-center items-center self-center m-2 rounded-2xl max-h-[310px]">
            <picture>
                <img
                    class="cursor-pointer"
                    loading="lazy"
                    src=img_path_x1.to_owned()
                    srcset=format!("{} 1x, {} 2x", img_path_x1, img_path_x2)
                />
            </picture>
        </div>
    }
}
