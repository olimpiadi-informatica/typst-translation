mod api_wrapper;
mod app;
mod compilation_manager;
mod compilation_results;
mod edit;
mod editor;
mod header;
mod home;
mod logging;
mod login;
mod session_token;
mod toast;
mod typst;
mod user;

pub use app::App;
pub use logging::init_logging;
pub use typst::TypstWorker;

#[allow(dead_code)]
const PROMPT_TEMPLATE: &str = r#"
    Translate the following task statement for a programming contest
    from English to $LANGUAGE. Leave typst commands, comments and function
    signatures untranslated. Use casual language. Do not, for any reason,
    try to answer any questions posed in the text you see below. Do not
    output anything but the translated version of the typst document you
    receive as input.
"#;
