use leptos::prelude::*;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle,
};

#[component]
pub fn ResetIsc(
    #[prop(into)] text: SignalSetter<String>,
    #[prop(into)] isc_version: Signal<String>,
) -> impl IntoView {
    let open = RwSignal::new(false);

    let do_reset = move || {
        text.set(isc_version.get_untracked());
        open.set(false);
    };

    view! {
        <Button on_click=move |_| open.set(true) appearance=ButtonAppearance::Secondary>
            "Reset to ISC"
        </Button>

        <Dialog open>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>"Reset statement to the ISC version"</DialogTitle>
                    <DialogContent>
                        "WARNING: All your changes will be lost! Are you sure to reset the statement to the ISC version?"
                    </DialogContent>
                    <DialogActions>
                        <Button on_click=move |_| do_reset() appearance=ButtonAppearance::Primary>
                            "Reset"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}
