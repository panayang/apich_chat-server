
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, web,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
    task::{Context, Poll},
};

use crate::auth::decode_jwt;


pub struct Authentication;

impl<S> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service: Rc::new(service) }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let auth_header = req.headers().get("Authorization");

            if let Some(header_value) = auth_header {
                if let Ok(header_str) = header_value.to_str() {
                    if header_str.starts_with("Bearer ") {
                        let token = header_str.trim_start_matches("Bearer ");
                        let secret_key = req.app_data::<web::Data<String>>().unwrap().to_string();

                        match decode_jwt(token, &secret_key) {
                            Ok(claims) => {
                                req.extensions_mut().insert(claims);
                                let res = svc.call(req).await?;
                                return Ok(res);
                            }
                            Err(_) => {
                                return Ok(req.into_response(HttpResponse::Unauthorized().finish()));
                            }
                        }
                    }
                }
            }
            Ok(req.into_response(HttpResponse::Unauthorized().finish()))
        })
    }
}
