use leptos::prelude::*;
use thaw::{FileList, Upload, UploadDragger};

use crate::header::Header;

#[component]
pub fn ImportTaskPage() -> impl IntoView {
    let owner = Owner::current().unwrap();
    let custom_request = move |_file_list: FileList| {
        owner.with(|| {
            // TODO
        });
    };

    view! {
        <Header title="Import Task" />
        <Upload custom_request multiple=true accept="application/zip">
            <UploadDragger>"Import tasks"</UploadDragger>
        </Upload>
    }
}
