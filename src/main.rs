// had read Date: 3.10
//http服务入口
use actix_web::{web, App, HttpServer};
mod scaling;
use crate::scaling::{service_query::ExternalServiceQuery, alert_handler};
use reqwest::Url;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化 ServiceQuery
    let service_query = ExternalServiceQuery::new(
       Url::parse("http://localhost:8080").unwrap() ,Some(String::from("token") )  );

    // 启动 HTTP 服务
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service_query.clone()))
            .service(
                web::resource("/system/alert")
                    .to(alert_handler::handle_alert)
            )
    })
    .bind("0.0.0.0:8080")?//监听地址
    .run()
    .await
}