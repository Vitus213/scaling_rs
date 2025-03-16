//alertmanger 处理器实现
// alert_handler.rs
use actix_web::{web, HttpResponse, Responder};
use crate::metrics::prometheus::PrometheusAlert;
use crate::scaling::service_query::ServiceQuery;
use crate::scaling::scaling_error::ScalingError;

pub async fn handle_alert(
    payload: actix_web::web::Json<PrometheusAlert>,    //接受报警数据，alertmanger发送webhook,json形式
    service_query: web::Data<dyn ServiceQuery>, //注入接口
    namespace: String,
) -> impl Responder {
    let mut errors :Vec<String> = Vec::new();//最后把scalingerror转为字符串返回
    
    for alert in &payload.alerts {
        log::info!("Processing alert: {:?}", alert);
        if let Some(function_name) = alert.labels.get_function_name() {
            match scale_service(
                &function_name,
                service_query.get_ref(),
                "None",//待定，之后补充
                &namespace
            ).await {
                Ok(_) => {},
                Err(e) => errors.push(e.to_string()),
            }
        }
    }
    
    if errors.is_empty() {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().body(errors.join("\n"))
    }
}

async fn scale_service(

    function_name: &str,
    service_query: &dyn ServiceQuery,
    status: &str,
    namespace: &str,
) -> Result<(), ScalingError> {
    //检查非空
    if function_name.is_empty(){
        return Ok(());
    };
    let resp = service_query.get_replicas(function_name, namespace).await?;
    
    let new_replicas = crate::scaling::service_query::calculate_replicas(
        status,
        resp.replicas,
        resp.min_replicas,
        resp.max_replicas,
        resp.scaling_factor
    );
      // 添加缩放日志（无论是否实际更新）
    log::info!(
        "[Scale] function={} {} => {}",
        function_name,
        resp.replicas,
        new_replicas
    );

    if new_replicas != resp.replicas {
        service_query.set_replicas(function_name, namespace, new_replicas).await?;
    }
    
    Ok(())
}