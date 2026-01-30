#[macro_export]
macro_rules! with_nocache {
    ($res:expr) => {{
        // Convert the provided value into an axum Response
        let mut resp = ::axum::response::IntoResponse::into_response($res);

        // Insert cache-control headers to avoid caching of auth-dependent pages/redirects
        resp.headers_mut().insert(
            ::http::header::CACHE_CONTROL,
            ::http::HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        );
        // Old user agents/proxies
        resp.headers_mut().insert(
            ::http::header::PRAGMA,
            ::http::HeaderValue::from_static("no-cache"),
        );
        // Tell shared caches the response depends on Cookie
        resp.headers_mut().insert(
            ::http::header::VARY,
            ::http::HeaderValue::from_static("Cookie"),
        );

        resp
    }};
}

#[macro_export]
macro_rules! with_nocache_ok {
    ($res:expr) => {{ Ok($crate::with_nocache!($res)) }};
}
