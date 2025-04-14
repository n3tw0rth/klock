use crate::error::Result;

trait Clock {
    async fn new() -> Self;
    async fn log() -> Result<()>;
}
