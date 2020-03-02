use futures::{Future};
use http::{self, HeaderMap, Request, Response, StatusCode};
use tower_service::Service;
use log::debug;
use std::pin::Pin;
use tonic::body::BoxBody;
use hyper::Body;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::cors::{CorsBuilder, CorsService};
use tonic::transport::NamedService;
use futures_util::future::TryFutureExt;
use futures_util::future::FutureExt;
use pretty_hex::*;

/// Decorates a service, providing an implementation of the CORS specification.
#[derive(Debug, Clone)]
pub struct ProxyService<S> {
    inner: CorsService<S>,
}
impl<S> NamedService for ProxyService<S> { 
    const NAME: &'static str = "proxy.GRPC-WebProxy";
}
impl<S> ProxyService<S> { 
    pub fn new(inner: S, cors_config: CorsBuilder) -> Self { 
        Self { 
            inner: CorsService::new(inner,Arc::new(cors_config.into_config()))
        }
    }   
}
impl<S> Service<Request<Body>> for ProxyService<S>
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
        //TODO: Remove me
        println!("\n\n\nRequest: {:?}", req);
       
        //If it's a HTTP/2 request, let it through.
        if req.version() == http::Version::HTTP_2 { 
            return self.inner.call(req);
        }
        
        //Error it the request is not Http/2 or Http1/1
        if req.version() != http::Version::HTTP_11 {
            return Box::pin(async { 
                let response = http::Response::builder().status(500).body(BoxBody::empty()).unwrap();
                Ok(response)
            });
        }

        //Handle CORs requests. 
        //TODO: Use a config for CORs
        if *req.method() == http::Method::OPTIONS
        {
            let origin = req.headers().get("origin").unwrap().to_str().unwrap().to_string(); 
            let fut = async move { 
                let response = http::Response::builder()
                    .header("access-control-allow-origin", origin)
                    .header("access-control-allow-headers", "keep-alive,user-agent,cache-control,content-type,content-transfer-encoding,custom-header-1,x-accept-content-transfer-encoding,x-accept-response-streaming,x-user-agent,x-grpc-web,grpc-timeout")
                    .header("access-control-expose-headers", "custom-header-1,grpc-status,grpc-message")
                    .header("access-control-allow-methods", "GET, PUT, DELETE, POST, OPTIONS")
                    .body(BoxBody::empty()).unwrap(); 

                //Modify the body
                println!("\nResponse: {:?}", response);
                Ok(response)
            };
            return Box::pin(fut);
        }
        //Update the version
        let version = req.version_mut(); 
        *version = http::Version::HTTP_2;

        //Get headers and transform stuff. 
        let (mut parts, body): (_, Body) = req.into_parts();
        
        //let content_type = parts.headers.get("content-type").unwrap().to_str().unwrap(); 
        let user_agent = parts.headers.get("x-user-agent").unwrap().to_str().unwrap(); 
        let origin = parts.headers.get("origin").unwrap().to_str().unwrap().to_string(); 

        parts.headers.insert("user-agent", user_agent.parse().unwrap());

        
        let body = hyper::body::Body::wrap_stream(hyper::body::to_bytes(body).and_then(|x|{
            let decoded = base64::decode_config(&x, base64::STANDARD).unwrap();
            println!("Body Decoded: {}", decoded.hex_dump());
            futures_util::future::ok(bytes::Bytes::from(decoded))
        }).into_stream());

        let req = http::Request::from_parts(parts, body);
        
        println!("\nTransformed: {:?}", req);
        
        let fut = self.inner.call(req);
        let fut = async move { 
            let mut response: http::Response<BoxBody> = fut.await.unwrap(); 
            //Modify the body
            let version = response.version_mut(); 
            *version = http::Version::HTTP_11;

            let (mut parts, body) = response.into_parts(); 

            let body = hyper::body::Body::wrap_stream(hyper::body::to_bytes(body).and_then(|x: bytes::Bytes|{
                println!("Response Body: \n {}", x.as_ref().hex_dump());
                let encoded = base64::encode_config(&x, base64::STANDARD);
                
                futures_util::future::ok(bytes::Bytes::from(encoded))
            }).into_stream()); 
            parts.headers.insert("grpc-accept-encoding", "identity".parse().unwrap());
            parts.headers.insert("grpc-encoding", "identity".parse().unwrap());
            parts.headers.insert("content-type", "application/grpc-web-text+proto".parse().unwrap());
            parts.headers.insert("access-control-allow-origin", origin.parse().unwrap());
            parts.headers.insert("access-control-expose-headers", "custom-header-1,grpc-status,grpc-message".parse().unwrap());
            parts.headers.insert("server", "mine".parse().unwrap());
            let response = http::Response::from_parts(parts, BoxBody::map_from(body));
            println!("\nResponse: {:?}", response);
            Ok(response)
        };
        Box::pin(fut)
        
    }
}

#[derive(Clone)]
struct ProxyConfig { 

}