use super::{Config, CorsResource};

use futures::{Future};

use http::{self, HeaderMap, Request, Response, StatusCode};
use tower_service::Service;
use log::debug;
use std::pin::Pin;
use tonic::body::BoxBody;
use hyper::Body;
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::transport::NamedService;

/// Decorates a service, providing an implementation of the CORS specification.
#[derive(Debug, Clone)]
pub struct CorsService<S> {
    pub inner: S,
    config: Arc<Config>,
}

impl<S> CorsService<S> {
    pub fn new(inner: S, config: Arc<Config>) -> CorsService<S> {
        CorsService { inner: inner, config }
    }
}

/*

  /// Create a router with the `S` typed service as the first service.
    ///
    /// This will clone the `Server` builder and create a router that will
    /// route around different services.
    pub fn add_service<S>(&mut self, svc: S) -> Router<S, Unimplemented>
    where
        S: Service<Request<Body>, Response = Response<BoxBody>>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<crate::Error> + Send,
    {
        Router::new(self.clone(), svc)
    }
*/

impl<S> Service<Request<Body>> for CorsService<S>
where
    S: Service<Request<Body>, Response = Response<BoxBody>> + Send + Clone,
    S::Future: Send + 'static,
    S::Error: Into<crate::Error> + 'static,
{
    type Response = Response<BoxBody>;
    type Error = crate::Error;
    //type Future = MapErr<Instrumented<S::Future>, fn(S::Error) -> crate::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
  

        let state = self.config.process_request(&req);
        let uri = req.uri().to_string();
        let version = req.version();

        let response_future = self.inner.call(req);
        
        let fut = async move { 
            //If it's a HTTP/2 request, let it through.
            if version == http::Version::HTTP_2 { 
                let mut response = response_future.await.ok().unwrap();
                return Ok(response);
            }
            match state { 
                Ok(CorsResource::Preflight(headers)) => {
                    let mut response = http::Response::new(BoxBody::empty());
                    *response.status_mut() = StatusCode::NO_CONTENT;
                    *response.headers_mut() = headers;
                    Ok(response)
                },
                Ok(CorsResource::Simple(headers)) => {
                    let mut response = response_future.await.ok().unwrap();
                    //let mut response = http::Response::new(BoxBody::empty());
                    response.headers_mut().extend(headers);
                    Ok(response)
                }
                Err(e) => {
                    debug!("CORS request to {} is denied: {:?}", uri, e);
                    let mut response = http::Response::new(BoxBody::empty());
                    *response.status_mut() = StatusCode::FORBIDDEN;
                    Ok(response)
                }
            }
        };
        Box::pin(fut)
    }
}
/*
impl<S> Service for CorsService<S>
where
    S: HttpService,
{
    type Request = Request<S::RequestBody>;
    type Response = Response<Option<S::ResponseBody>>;
    type Error = S::Error;
    type Future = CorsFuture<S::Future>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.inner.poll_http_ready()
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        let inner = match self.config.process_request(&request) {
            Ok(CorsResource::Preflight(headers)) => CorsFutureInner::Handled(Some(headers)),
            Ok(CorsResource::Simple(headers)) => {
                CorsFutureInner::Simple(self.inner.call_http(request), Some(headers))
            }
            Err(e) => {
                debug!("CORS request to {} is denied: {:?}", request.uri(), e);
                CorsFutureInner::Handled(None)
            }
        };

        CorsFuture(inner)
    }
}


#[derive(Debug)]
pub struct CorsFuture<F>(CorsFutureInner<F>);

impl<F, ResponseBody> Future for CorsFuture<F>
where
    F: Future<Output= Result<http::Response<ResponseBody>, ()>>,
{

    type Output = Result<http::Response<Option<ResponseBody>>, ()>; 

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { 
        self.0.poll(cx)
    }
}

#[derive(Debug)]
enum CorsFutureInner<F> {
    Simple(F, Option<HeaderMap>),
    Handled(Option<HeaderMap>),
}

impl<F, ResponseBody> Future for CorsFutureInner<F>
where
    F: Future<Output= Result<http::Response<ResponseBody>, ()>>,
{
    type Output = Result<http::Response<Option<ResponseBody>>, ()>; 
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { 
        use self::CorsFutureInner::*;

        match self {
            Simple(f, headers) => {
                let mut response = try_ready!(f.poll());
                let headers = headers.take().expect("poll called twice");
                response.headers_mut().extend(headers);
                Ok(Async::Ready(response.map(Some)))
            }
            Handled(headers) => {
                let mut response = http::Response::new(None);
                *response.status_mut() = StatusCode::FORBIDDEN;

                if let Some(headers) = headers.take() {
                    *response.status_mut() = StatusCode::NO_CONTENT;
                    *response.headers_mut() = headers;
                }

                Ok(Async::Ready(response))
            }
        }
    }
}
*/