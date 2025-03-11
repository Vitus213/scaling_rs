// scale_zero.rs

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};

pub struct ScaleFromZeroMiddleware<S> {
    service: S,
    scaler: Arc<dyn FunctionScaler>,
}

impl<S> Service<ServiceRequest> for ScaleFromZeroMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
{
    async fn call(&self, req: ServiceRequest) -> Result<ServiceResponse, Error> {
        // 从请求路径提取函数名
        let function_name = req.match_info().get("name").unwrap();
        
        // 检查当前副本数
        let scaler = self.scaler.clone();
        let result = scaler.scale_from_zero(function_name).await;
        
        match result {
            Ok(true) => self.service.call(req).await,
            Ok(false) => Ok(ServiceResponse::new(
                req.into_parts().0,
                HttpResponse::ServiceUnavailable().finish(),
            )),
            Err(e) => {
                log::error!("Scaling error: {}", e);
                Ok(ServiceResponse::new(
                    req.into_parts().0,
                    HttpResponse::InternalServerError().finish(),
                ))
            }
        }
    }
}