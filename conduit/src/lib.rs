#![warn(rust_2018_idioms)]

use std::error::Error;
use std::io::Read;

pub use http::{
    header, Extensions, HeaderMap, Method, Request, Response, StatusCode, Uri, Version,
};

pub type ResponseResult<Error> = Result<Response<Body>, Error>;
pub type HttpResult = ResponseResult<http::Error>;

pub type BoxError = Box<dyn Error + Send>;
pub type HandlerResult = Result<Response<Body>, BoxError>;

/// A type representing a `Response` body.
///
/// This type is intended exclusively for use as part of a `Response<Body>`.
/// Each conduit server provides its own request type that implements
/// `RequestExt` which provides the request body as a `&'a mut dyn Read`.
pub enum Body {
    Static(&'static [u8]),
    Owned(Vec<u8>),
}

impl Body {
    /// Create a new `Body` from an empty static slice.
    pub fn empty() -> Self {
        Self::from_static(b"")
    }

    /// Create a new `Body` from the provided static byte slice.
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self::Static(bytes)
    }

    /// Create a new `Body` by taking ownership of the provided bytes.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self::Owned(bytes)
    }
}

/// A helper to convert a concrete error type into a `Box<dyn Error + Send>`
///
/// # Example
///
/// ```
/// # use std::error::Error;
/// # use conduit::{box_error, Body, Response};
/// # let _: Result<Response<Body>, Box<dyn Error + Send>> =
/// Response::builder().body(Body::empty()).map_err(box_error);
/// ```
pub fn box_error<E: Error + Send + 'static>(error: E) -> BoxError {
    Box::new(error)
}

pub trait RequestExt {
    /// The version of HTTP being used
    fn http_version(&self) -> Version;

    /// The request method, such as GET, POST, PUT, DELETE or PATCH
    fn method(&self) -> &Method;

    /// The request URI
    fn uri(&self) -> &Uri;

    /// The byte-size of the body, if any
    fn content_length(&self) -> Option<u64>;

    /// The request's headers, as conduit::Headers.
    fn headers(&self) -> &HeaderMap;

    /// A Reader for the body of the request
    ///
    /// # Blocking
    ///
    /// The returned value implements the blocking `Read` API and should only
    /// be read from while in a blocking context.
    fn body(&mut self) -> &mut dyn Read;

    /// A readable map of extensions
    fn extensions(&self) -> &Extensions;

    /// A mutable map of extensions
    fn mut_extensions(&mut self) -> &mut Extensions;
}

/// A Handler takes a request and returns a response or an error.
/// By default, a bare function implements `Handler`.
pub trait Handler: Sync + Send + 'static {
    fn call(&self, request: &mut dyn RequestExt) -> HandlerResult;
}

impl<F, E> Handler for F
where
    F: Fn(&mut dyn RequestExt) -> ResponseResult<E> + Sync + Send + 'static,
    E: Error + Send + 'static,
{
    fn call(&self, request: &mut dyn RequestExt) -> HandlerResult {
        (*self)(request).map_err(box_error)
    }
}