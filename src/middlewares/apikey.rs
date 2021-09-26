use std::pin::Pin;
use std::task::{Context, Poll};
use std::rc::Rc;
use std::cell::RefCell;

use actix_web::{Error, http::HeaderName, ResponseError, http::StatusCode, http::header::ToStrError};
use actix_web::dev::{ServiceRequest, ServiceResponse, Service, Transform};
use std::future::{Ready, Future, ready};

const API_KEY_HEADER_NAME : &[u8] = b"X-CB-API-KEY";

#[derive(Debug)]
pub enum ApiKeyError {
    Invalid,
    InvalidEncoding
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            ApiKeyError::Invalid => write!(f, "Invalid API key"),
            ApiKeyError::InvalidEncoding => write!(f, "API keys are ASCII values")
        }
    }
}

impl ResponseError for ApiKeyError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.


pub struct ApiKeyService<F> where F : FnMut(Option<&str>) -> bool {
    validator : Rc<RefCell<F>>
}

impl<F> ApiKeyService<F> where F : FnMut(Option<&str>) -> bool {
    pub fn from_validator(f : F) -> ApiKeyService<F> {
        ApiKeyService {
            validator : Rc::new(RefCell::from(f))
        }
    }
}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B, F> Transform<S, ServiceRequest> for ApiKeyService<F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    F: FnMut(Option<&str>) -> bool
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddleware<S, F>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let new_validator = self.validator.clone();
        ready(Ok(ApiKeyMiddleware { service, validator: new_validator }))
    }
}

pub struct ApiKeyMiddleware<S, F> where F : FnMut(Option<&str>) -> bool {
    service: S,
    validator : Rc<RefCell<F>>
}

impl<S, B, F> Service<ServiceRequest> for ApiKeyMiddleware<S, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    F: FnMut(Option<&str>) -> bool
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let api_key_hv_opt = req.headers().get(HeaderName::from_bytes(API_KEY_HEADER_NAME).unwrap());
        let api_key_opt_result : Result<Option<&str>, ToStrError> = api_key_hv_opt.map(|api_key_hv| api_key_hv.to_str()).transpose();
        if api_key_opt_result.is_err() {
            return Box::pin(ready(Err(ApiKeyError::InvalidEncoding.into())));
        }
        let api_key_opt = api_key_opt_result.unwrap();
        if !self.validator.borrow_mut()(api_key_opt) {
            return Box::pin(ready(Err(ApiKeyError::Invalid.into())));
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}