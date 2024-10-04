mod sync;

pub async fn run() {
    sync::sync().await;
}
