use frontend::{TypstWorker, init_logging};
use gloo_worker::Registrable;

fn main() {
    init_logging();

    TypstWorker::registrar().register();
}
