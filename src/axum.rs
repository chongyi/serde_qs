use std::ops::Deref;
use std::fmt::{self, Display};

use axum::extract::RequestParts;
use axum::{http, Error, BoxError};
use axum::response::{IntoResponse, Response};
use axum_framework as axum;
use axum::extract::FromRequest;
use serde::de::DeserializeOwned;

/// Extractor that deserializes query strings into some type.
///
/// `T` is expected to implement [`serde::Deserialize`].
///
/// # Example
///
/// ```rust,no_run
/// use axum::{
///     routing::get,
///     Router,
/// };
/// use serde::Deserialize;
/// use serde_qs::axum::QsQuery;
///
/// #[derive(Deserialize)]
/// struct Pagination {
///     page: usize,
///     per_page: usize,
/// }
///
/// // This will parse query strings like `?page=2&per_page=30` into `Pagination`
/// // structs.
/// async fn list_things(pagination: QsQuery<Pagination>) {
///     let pagination: Pagination = pagination.0;
///
///     // ...
/// }
///
/// let app = Router::new().route("/list_things", get(list_things));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// If the query string cannot be parsed it will reject the request with a `422
/// Unprocessable Entity` response.
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
#[derive(Debug, Clone, Copy, Default)]
pub struct QsQuery<T>(pub T);

impl<T> Deref for QsQuery<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Display> Display for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}


#[async_trait::async_trait]
impl<T, B> FromRequest<B> for QsQuery<T>
where
    T: DeserializeOwned,
    B: Send,
{
    type Rejection = QsQueryRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let query = req.uri().query().unwrap_or_default();
        let value = crate::from_str(query)
            .map_err(QsQueryRejection::new::<T, _>)?;
        Ok(QsQuery(value))
    }
}


/// Rejection type for extractors that deserialize query strings if the input
/// couldn't be deserialized into the target type.
#[derive(Debug)]
pub struct QsQueryRejection {
    error: Error,
    type_name: &'static str,
}

impl QsQueryRejection {
    pub(super) fn new<T, E>(error: E) -> Self
    where
        E: Into<BoxError>,
    {
        QsQueryRejection {
            error: Error::new(error),
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl IntoResponse for QsQueryRejection {
    fn into_response(self) -> Response {
        (http::StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
    }
}

impl std::fmt::Display for QsQueryRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to deserialize query string. Expected something of type `{}`. Error: {}",
            self.type_name, self.error,
        )
    }
}

impl std::error::Error for QsQueryRejection {}