use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct GreetingParams {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GreetingRequest {
    id: String,
    jsonrpc: String,
    method: String,
    params: GreetingParams,
}

#[derive(Debug, Serialize)]
struct GreetingResponse {
    id: String,
    jsonrpc: String,
    result: GreetingResult,
}

#[derive(Debug, Serialize)]
struct GreetingResult {
    greeting: String,
}

async fn json_rpc_handler(item: web::Json<GreetingRequest>) -> HttpResponse {
    let obj = GreetingResponse {
        id: item.id.clone(),
        jsonrpc: item.jsonrpc.clone(),
        result: GreetingResult {
            greeting: format!("Hello, {}!", item.params.name),
        },
    };
    HttpResponse::Ok().json(obj)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096))
            .service(web::resource("/").route(web::post().to(json_rpc_handler)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use crate::{json_rpc_handler, GreetingParams, GreetingRequest};
    use actix_web::{body::to_bytes, dev::Service, http, test, web, App};

    #[actix_web::test]
    async fn test_happy_path() {
        let app = test::init_service(
            App::new().service(web::resource("/").route(web::post().to(json_rpc_handler))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&GreetingRequest {
                id: "00000000-0000-0000-0000-000000000000".to_owned(),
                jsonrpc: "2.0".to_owned(),
                method: "greeting".to_owned(),
                params: GreetingParams {
                    name: "Oliver".to_owned(),
                },
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(
            body_bytes,
            r##"{"id":"00000000-0000-0000-0000-000000000000","jsonrpc":"2.0","result":{"greeting":"Hello, Oliver!"}}"##
        );
    }
}
