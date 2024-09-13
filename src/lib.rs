use crate::api::schema::CustomerDetails;
use bytes::{Buf, Bytes};
use custom_logger::*;
use http_body_util::{BodyExt, Full};
use hyper::body::Body;
use hyper::{header, Error, Method, Request, Response, StatusCode};
use std::fmt::Debug;

pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

mod api;

/// handler - this is the main entry point used in the unikernel serverless framework
// The impmentation is completely flexible, just ensure
// the function name, input and output parameters don't change
pub async fn process_handler<H: hyper::body::Body>(
    log: &Logging,
    req: Request<H>,
) -> Result<Response<BoxBody>, Error>
where
    <H as Body>::Error: Debug,
{
    log.info(&format!("processing request {:?}", req.method()));
    match (req.method(), req.uri().path()) {
        // from POST
        (&Method::POST, "/publish") => {
            let payload = req.collect().await;
            let data: CustomerDetails =
                serde_json::from_reader(payload.unwrap().aggregate().reader()).unwrap();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(full(format!("well hello {}", data.name)))
                .unwrap();
            Ok(response)
        }
        // health endpoint
        (&Method::GET, "/isalive") => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(full("ok"))
                .unwrap();
            Ok(response)
        }
        // all other routes and methods
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(full("error not implemented"))
                .unwrap();
            Ok(response)
        }
    }
}

// full - utility to build BoxBody
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[cfg(test)]
mod tests {
    // this brings everything from parent's scope into this scope
    use super::*;

    // used to ensure we handle async calls (as no async i.e await keyword can be used in testing)
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e).unwrap()
        };
    }

    #[test]
    fn test_process_handler_isalive_pass() {
        let log = &Logging {
            log_level: Level::TRACE,
        };
        let req = Request::builder()
            .uri("/isalive")
            .method(Method::GET)
            .body("".to_string())
            .unwrap();

        let res = aw!(process_handler(log, req.clone()));
        let status = res.status().clone();
        let data_hld = aw!(res.into_body().boxed().collect());
        let data = data_hld.to_bytes();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(Bytes::from("ok"), data);
    }
    #[test]
    fn test_process_handler_publish_pass() {
        let log = &Logging {
            log_level: Level::TRACE,
        };
        let publish_data = r#"
        {
            "email":"test@test.com",
            "name": "john",
            "surname": "doe",
            "mobile": "12345678",
            "id": "987654321"
        }
        "#;
        let req = Request::builder()
            .uri("/publish")
            .method(Method::POST)
            .body(publish_data.to_string())
            .unwrap();
        let res = aw!(process_handler(log, req.clone()));
        let status = res.status().clone();
        let data_hld = aw!(res.into_body().boxed().collect());
        let data = data_hld.to_bytes();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(Bytes::from("well hello john"), data);
    }
    #[test]
    fn test_process_handler_error_pass() {
        let log = &Logging {
            log_level: Level::TRACE,
        };
        let req = Request::builder()
            .uri("/error")
            .method(Method::PUT)
            .body("error".to_string())
            .unwrap();
        let res = aw!(process_handler(log, req.clone()));
        let status = res.status().clone();
        let data_hld = aw!(res.into_body().boxed().collect());
        let data = data_hld.to_bytes();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(Bytes::from("error not implemented"), data);
    }
}
