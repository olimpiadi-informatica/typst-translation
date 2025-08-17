use frontend::{logging::init_logging, typst::TypstWorker};
use gloo_worker::Registrable;

fn main() {
    init_logging();

    TypstWorker::registrar().register();
}
