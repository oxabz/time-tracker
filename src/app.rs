use leptos::*;
use wasm_bindgen::prelude::*;

use crate::{components::{actions::Actions, reporting::Reporting, statistics::Statistics, timeline::Timeline}, notifications::Notifications};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}


#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container mx-auto p-4 h-screen flex flex-col gap-4 ">
            <Notifications/>
            <div class="bg-base-200 p-6 items-center rounded-lg">
                <Timeline/>
            </div>
            <Reporting/>
            <Statistics/>
            <Actions/>
        </main>
    }
}
