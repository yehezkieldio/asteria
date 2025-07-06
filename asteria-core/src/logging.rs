use tracing_subscriber::fmt;

pub fn init_logging() {
    let subscriber: fmt::SubscriberBuilder = fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    subscriber.init();
}
