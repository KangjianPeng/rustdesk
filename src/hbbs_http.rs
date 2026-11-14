mod http_client;
pub mod record_upload;
pub mod sync;
pub use http_client::{
    create_http_client_async, create_http_client_with_url, create_http_client_with_url_strict,
    get_url_for_tls,
};
