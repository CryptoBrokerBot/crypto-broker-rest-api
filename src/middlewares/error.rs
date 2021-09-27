use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::{Error, ResponseError, http::StatusCode};
use actix_web::dev::{ServiceRequest, ServiceResponse, Service, Transform};
use std::future::{Ready, Future, ready};


#[derive(Debug)]
enum ErrorHandlerWrappedError {
    Message(String)
}

impl std::fmt::Display for ErrorHandlerWrappedError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            ErrorHandlerWrappedError::Message(s) => {
                let clean_str = s.replace("\\", "\\\\").replace("\"", "\\\"");
                write!(f, "{}", "{\"success\":false,\"message\":\"")?;
                write!(f, "{}", clean_str)?;
                write!(f, "{}", "\"}")?;
                Ok(())
            }
        }
    }
}

impl ResponseError for ErrorHandlerWrappedError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct ErrorHandlerService;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for ErrorHandlerService
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorHandlerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ErrorHandlerMiddleware { service }))
    }
}

pub struct ErrorHandlerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ErrorHandlerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let response_result = fut.await;
            match response_result {
                Ok(response) => Ok(response),
                Err(err) => Err(ErrorHandlerWrappedError::Message(format!("{}", err)).into())
            }
        })
    }
}
