use either::Either;

pub struct Response<T> {
    pub result: T,
}

/// ErrResponse contains a parsed error, but if it wasn't able to parse the
/// response it will contain the http body as-is.
pub struct ErrResponse<T> {
    pub result: Either<T, String>,
}

pub struct PagedResponse<T> {
    pub result: Vec<T>,
    pub total: u64,
}
