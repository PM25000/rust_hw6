use axum::{
    extract::{Path, State},
    response::IntoResponse,
    response::Response,
    routing::post,
    Form, Json, Router,
};
use axum_macros::debug_handler;
use lazy_static::lazy_static;
use pilota::FastStr;
use reqwest::StatusCode;
use std::{
    io::{self, BufRead, Write},
    net::SocketAddr,
    str::FromStr,
};
use volo_example::LogLayer;

type Client = volo_gen::volo::example::ItemServiceClient;

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:10818".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            // .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
}

async fn get_item(key: FastStr) -> volo_gen::volo::example::GetItemResponse {
    let req = volo_gen::volo::example::GetItemRequest { key };
    let resp = CLIENT.get_item(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

async fn set_item(key: FastStr, value: FastStr) -> volo_gen::volo::example::SetItemResponse {
    let req = volo_gen::volo::example::SetItemRequest {
        kv: {
            let mut kv = volo_gen::volo::example::Kv::default();
            kv.key = key;
            kv.value = value;
            kv
        },
    };
    let resp = CLIENT.set_item(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

async fn delete_item(keys: Vec<FastStr>) -> volo_gen::volo::example::DeleteItemResponse {
    let req = volo_gen::volo::example::DeleteItemRequest { keys };
    let resp = CLIENT.delete_item(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

async fn ping(msg: Option<String>) -> volo_gen::volo::example::PingResponse {
    let req = volo_gen::volo::example::PingRequest {
        message: msg.map(|s| FastStr::from(s)),
    };
    let resp = CLIENT.ping(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

async fn handle_ping() -> Response {
    let resp = ping(None).await;
    (StatusCode::OK, resp.message.to_string()).into_response()
}

#[derive(serde::Serialize)]
struct GetItemResponse {
    value: String,
}

#[debug_handler]
async fn handle_get_item(
    Path(key): Path<String>,
    State(cli): State<Client>,
) -> (StatusCode, Json<GetItemResponse>) {
    let resp = cli
        .get_item(volo_gen::volo::example::GetItemRequest {
            key: FastStr::from(key),
        })
        .await;
    match resp {
        Ok(info) => (
            StatusCode::OK,
            Json(GetItemResponse {
                value: info.value.to_string(),
            }),
        ),
        Err(e) => {
            tracing::error!("{:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetItemResponse {
                    value: String::from("error"),
                }),
            )
        }
    }
}

#[derive(serde::Deserialize)]
struct Kv {
    key: String,
    value: String,
}

#[debug_handler]
async fn handle_set_item(State(cli): State<Client>, Json(kv): Json<Kv>) -> Response {
    let resp = cli
        .set_item(volo_gen::volo::example::SetItemRequest {
            kv: volo_gen::volo::example::Kv {
                key: FastStr::from(kv.key),
                value: FastStr::from(kv.value),
            },
        })
        .await;
    match resp {
        Ok(info) => (StatusCode::OK, "set ok").into_response(),
        Err(e) => {
            tracing::error!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "set error").into_response()
        }
    }
}

#[derive(serde::Deserialize)]
struct DelParam {
    keys: Vec<String>,
}

async fn handle_delete_item(State(cli): State<Client>, Json(delParam): Json<DelParam>) -> Response {
    let resp = cli
        .delete_item(volo_gen::volo::example::DeleteItemRequest {
            keys: delParam
                .keys
                .into_iter()
                .map(|s| FastStr::from(s))
                .collect(),
        })
        .await;
    match resp {
        Ok(info) => (StatusCode::OK, format!("delete {} items", info.count)).into_response(),
        Err(e) => {
            tracing::error!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "delete error").into_response()
        }
    }
}

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let resp = ping(None).await;

    assert_eq!(resp.message.to_ascii_lowercase(), FastStr::from(""));

    let app = Router::new()
        .route("/ping", post(handle_ping))
        .route(
            "/get/:key",
            post(handle_get_item).with_state(CLIENT.clone()),
        )
        .route("/set", post(handle_set_item).with_state(CLIENT.clone()))
        .route(
            "/delete",
            post(handle_delete_item).with_state(CLIENT.clone()),
        );

    let addr: SocketAddr = "127.0.0.1:10820".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    return;

    loop {
        print!("> ");
        io::stdout().flush().expect("failed to flush stdout");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read from stdin");

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let mut args = input.split_whitespace();
        let cmd = args.next().unwrap();
        let args = args.collect::<Vec<_>>();

        match cmd {
            "get" => {
                let key = args[0];
                let resp = get_item(String::from(key).into()).await;
                println!("{:?}", resp);
            }
            "set" => {
                let key = args[0];
                let value = args[1];
                let resp = set_item(String::from(key).into(), String::from(value).into()).await;
                println!("{:?}", resp);
            }
            "delete" => {
                let keys = args.iter().map(|s| String::from(*s).into()).collect();
                let resp = delete_item(keys).await;
                println!("{:?}", resp);
            }
            "ping" => {
                let msg = args.join(" ");
                let resp = if args.is_empty() {
                    ping(None).await
                } else {
                    ping(Some(msg)).await
                };
                println!("{:?}", resp);
            }
            "exit" => {
                break;
            }
            _ => {
                println!("unknown command: {}", cmd);
            }
        }
    }
}
