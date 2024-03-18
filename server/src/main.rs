use tracing::level_filters::LevelFilter;

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_max_level(if cfg!(debug_assertions) {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
