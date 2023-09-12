#![feature(impl_trait_in_assoc_type)]

use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use anyhow::{Ok, anyhow};
use lazy_static::lazy_static;
use pilota::FastStr;
use tracing_subscriber::fmt::format;
use volo_gen::volo::example::{
    DeleteItemResponse, ItemServicePingResultSend, ItemServiceRequestRecv, PingResponse,
    SetItemResponse,
};
use volo_thrift::ResponseError;
type Db = Arc<Mutex<HashMap<FastStr, FastStr>>>;

lazy_static! {
    static ref DB: Db = Arc::new(Mutex::new(HashMap::new()));
}
pub struct S;

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
    async fn get_item(
        &self,
        _req: volo_gen::volo::example::GetItemRequest,
    ) -> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError>
    {
        println!("get_item");
        println!("{}", _req.key.to_string());
        let db = DB.lock().unwrap();
        let value = db.get(&_req.key);
        match value {
            Some(v) => {
                let mut resp = volo_gen::volo::example::GetItemResponse::default();
                resp.value = v.clone();
                Ok(resp)
            }
            None => Ok(Default::default()),
        }
    }

    async fn post_item(
        &self,
        _req: volo_gen::volo::example::PostItemRequest,
    ) -> ::core::result::Result<volo_gen::volo::example::PostItemResponse, ::volo_thrift::AnyhowError>
    {
        println!("post_item");
        println!("{}", _req.name);
        Ok(Default::default())
    }

    async fn set_item(
        &self,
        _req: volo_gen::volo::example::SetItemRequest,
    ) -> ::core::result::Result<volo_gen::volo::example::SetItemResponse, ::volo_thrift::AnyhowError>
    {
        println!("set_item");
        println!("{}:{}", _req.kv.key.to_string(), _req.kv.value.to_string());
        let mut db = DB.lock().unwrap();
        db.insert(_req.kv.key, _req.kv.value);
        Ok(SetItemResponse {
            message: FastStr::from("OK"),
        })
    }

    async fn delete_item(
        &self,
        _req: volo_gen::volo::example::DeleteItemRequest,
    ) -> ::core::result::Result<
        volo_gen::volo::example::DeleteItemResponse,
        ::volo_thrift::AnyhowError,
    > {
        println!("delete_item");
        let mut db = DB.lock().unwrap();
        let mut count = 0;
        for k in _req.keys.clone() {
            if db.contains_key(&k) {
                db.remove(&k);
                count += 1;
            }
        }
        Ok(DeleteItemResponse { count })
    }

    async fn ping(
        &self,
        _req: volo_gen::volo::example::PingRequest,
    ) -> ::core::result::Result<volo_gen::volo::example::PingResponse, ::volo_thrift::AnyhowError>
    {
        println!("ping:");
        if let Some(v) = _req.message.clone() {
            println!("{}", v.to_string());
        } else {
            println!("PONG");
        }
        Ok(PingResponse {
            message: match _req.message {
                Some(v) => v.clone(),
                None => FastStr::from("PONG"),
            },
        })
    }
}

#[derive(Clone)]
pub struct LogService<S>(S);

// #[volo::service]
// impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
// where
//     Req: std::fmt::Debug + Send + 'static ,
//     S: Send + 'static + volo::Service<Cx, Req> + Sync,
//     S::Response: std::fmt::Debug,
//     S::Error: std::fmt::Debug,
//     Cx: Send + 'static,
// {
//     type Response = S::Response;

//     type Error = anyhow::Error;

//     type Future<'cx> = impl Future<Output = Result<S::Response, Self::Error>> + 'cx;

//     fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: Req) -> Self::Future<'cx>
//     where
//         's: 'cx,
//     {
//         async move {
//             let now = std::time::Instant::now();
//         	// tracing::debug!("Received request {:?}", &req);
// 			println!("{:?}", req);
// 			let req_str =format!("{:?}", req);
// 			let req_str = req_str.as_str();

// 			if req_str.starts_with("Ping") {
// 				println!("Ping");
// 				return Err(anyhow::Error::msg("Ping"));
// 			}

// 			let resp = self.0.call(cx, req).await;
// 			let resp = match resp {
// 				Result::Ok(v) => {
// 					Ok(v)
// 				}
// 				Err(_e) => {
// 					Err(anyhow::Error::msg("some err"))
// 				}

// 			};

// 			println!("{:?}", resp);

// 			// tracing::debug!("Sent response {:?}", &resp);
// 			tracing::info!("Request took {}ms", now.elapsed().as_millis());
// 			resp
//         }
//     }

// }

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static,
    anyhow::Error: Into<S::Error>,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let now = std::time::Instant::now();
        tracing::debug!("Received request {:?}", &req);

        let req_str = format!("{:?}", req);
        let req_str = req_str.as_str();

        if req_str.starts_with("Ping") {
            println!("Ping");
            println!("{}", req_str);
            if req_str.contains("message: None") {
                return Err(anyhow!("reject").into());
            }
        }

        let resp = self.0.call(cx, req).await;
        tracing::debug!("Sent response {:?}", &resp);
        tracing::info!("Request took {}ms", now.elapsed().as_millis());
        resp
    }
}
pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}
