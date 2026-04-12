use leptos::portal::Portal;
use leptos::prelude::*;

use crate::util::Icon;

#[component]
pub fn ResetIsc(
    #[prop(into)] text: SignalSetter<String>,
    #[prop(into)] isc_version: Signal<String>,
) -> impl IntoView {
    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();

    let do_reset = move |_| {
        text.set(isc_version.get_untracked());
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
    };

    let open_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.show_modal();
        }
    };

    let close_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
    };

    view! {
        <button class="btn btn-secondary btn-sm join-item" on:click=open_dialog>
            "Reset to ISC"
        </button>

        <Portal>
            <dialog node_ref=dialog_ref class="modal">
                <div class="modal-box">
                    <h3 class="font-bold text-lg">"Reset statement to the ISC version"</h3>
                    <div class="py-4">
                        <div class="alert alert-error">
                            <Icon icon=icondata::IoWarning />
                            <span>
                                "WARNING: All your changes will be lost! Are you sure to reset the statement to the ISC version?"
                            </span>
                        </div>
                    </div>
                    <div class="modal-action">
                        <button class="btn btn-error" on:click=do_reset>
                            "Reset"
                        </button>
                        <button class="btn" on:click=close_dialog>
                            "Cancel"
                        </button>
                    </div>
                </div>
                <form method="dialog" class="modal-backdrop">
                    <button>"close"</button>
                </form>
            </dialog>
        </Portal>
    }
}
