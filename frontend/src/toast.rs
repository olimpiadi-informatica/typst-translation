#[macro_export]
macro_rules! show_error {
    ($($arg: tt)*) => {
        use thaw::{Toast, ToastIntent, ToastOptions, ToastTitle, ToasterInjection};
        let message = format!($($arg)*);
        let toaster = ToasterInjection::expect_context();
        //thaw::LoadingBarInjection::expect_context().error();
        tracing::error!("{message}");
        toaster.dispatch_toast(
            || {
                leptos::view! {
                    <Toast>
                        <ToastTitle>{message}</ToastTitle>
                    </Toast>
                }
            },
            ToastOptions::default().with_intent(ToastIntent::Error),
        );
    };
}

#[macro_export]
macro_rules! show_success {
    ($($arg: tt)*) => {
        use thaw::{Toast, ToastIntent, ToastOptions, ToastTitle, ToasterInjection};
        let message = format!($($arg)*);
        let toaster = ToasterInjection::expect_context();
        tracing::info!("{message}");
        toaster.dispatch_toast(
            || {
                leptos::view! {
                    <Toast>
                        <ToastTitle>{message}</ToastTitle>
                    </Toast>
                }
            },
            ToastOptions::default().with_intent(ToastIntent::Success),
        );
    };
}
