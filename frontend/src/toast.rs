use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    // Info,
    // Warning,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: Uuid,
    pub kind: ToastKind,
    pub message: String,
}

#[derive(Clone, Copy)]
pub struct ToastContext {
    pub toasts: RwSignal<Vec<Toast>>,
}

impl ToastContext {
    pub fn push(&self, kind: ToastKind, message: String) {
        let id = Uuid::new_v4();
        self.toasts.update(|toasts| {
            toasts.push(Toast {
                id,
                kind,
                message: message.clone(),
            });
        });

        // Auto-remove toast after some time
        let toasts_signal = self.toasts;
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(5000).await;
            toasts_signal.update(|toasts| {
                toasts.retain(|t| t.id != id);
            });
        });
    }
}

#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toasts = RwSignal::new(Vec::<Toast>::new());
    let context = ToastContext { toasts };
    provide_context(context);

    view! {
        {children()}
        <div class="toast toast-end">
            <For each=move || toasts.get() key=|toast| toast.id let(toast)>
                <div class=format!(
                    "alert {}",
                    match toast.kind {
                        ToastKind::Success => "alert-success",
                        ToastKind::Error => "alert-error",
                    },
                )>
                    <span>{toast.message}</span>
                </div>
            </For>
        </div>
    }
}

#[macro_export]
macro_rules! show_error {
    ($($arg: tt)*) => {
        let message = format!($($arg)*);
        tracing::error!("{message}");
        if let Some(toaster) = leptos::prelude::use_context::<$crate::toast::ToastContext>() {
            toaster.push($crate::toast::ToastKind::Error, message);
        }
    };
}

#[macro_export]
macro_rules! show_success {
    ($($arg: tt)*) => {
        let message = format!($($arg)*);
        tracing::info!("{message}");
        if let Some(toaster) = leptos::prelude::use_context::<$crate::toast::ToastContext>() {
            toaster.push($crate::toast::ToastKind::Success, message);
        }
    };
}
