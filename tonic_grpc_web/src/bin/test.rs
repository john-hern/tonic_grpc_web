use tower_service::Service;

use http::{Request, Response};
use tonic::body::BoxBody;
use hyper::Body;
use std::pin::Pin;
use std::sync::Arc;
use tower_grpc_proxy::cors::{CorsService, CorsBuilder};
use tower_grpc_proxy::cors::{Config};

fn main() {
    println!("Hello, world!");
}



/*
impl<S> Service for ProxyService<S>
where
    S: Service<Request<Body>, Response = Response<BoxBody>> + Send,
    S::Future: Send + 'static,
    S::Error: Into<std::io::Error> + 'static,
{

    type Response = Response<BoxBody>;
    type Error = std::io::Error;
    //type Future = MapErr<Instrumented<S::Future>, fn(S::Error) -> crate::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;


    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.inner.poll_http_ready()
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        futures::future::ok(())
    }
}
*/