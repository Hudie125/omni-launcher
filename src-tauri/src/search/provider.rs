use super::SearchResultItem;
use std::future::Future;
use std::pin::Pin;
use tokio_util::sync::CancellationToken;

pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn search<'a>(
        &'a self,
        query: &'a str,
        cancel_token: &'a CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Vec<SearchResultItem>> + Send + 'a>>;
}
