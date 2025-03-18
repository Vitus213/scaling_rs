//核心 Trait 定义（ServiceQuery）
use super::scaling_error::ScalingError;
use async_trait::async_trait;
use reqwest::{StatusCode, Url};
use std::cmp::max;
use std::collections::HashMap;
const DEFAULT_MIN_REPLICAS: u64 = 1;
const DEFAULT_SCALING_FACTOR: u8 = 20;
const DEFAULT_MAX_REPLICAS: u64 = 100; // 默认最大副本数
#[async_trait]
//ServiceQuery接口实现
pub trait ServiceQuery {
    async fn get_replicas(
        &self,
        service: &str,
        namespace: &str,
    ) -> Result<ServiceQueryResponse, ScalingError>;

    async fn set_replicas(
        &self,
        service: &str,
        namespace: &str,
        count: u64,
    ) -> Result<(), ScalingError>;
}

#[derive(Debug, Clone)]
pub struct ServiceQueryResponse {
    //照抄
    pub replicas: u64,
    pub min_replicas: u64, //最小
    pub max_replicas: u64,
    pub scaling_factor: u8, //步长
    pub available_replicas: u64,
    pub annotations: std::collections::HashMap<String, String>,
}

//HTTP 客户端实现（对应 Go 的 ExternalServiceQuery）
// service_query.rs
#[derive(Clone)]
pub struct ExternalServiceQuery {
    client: reqwest::Client,
    base_url: Url,
    auth_token: Option<String>,
    include_usage: bool,
}

impl ExternalServiceQuery {
    pub fn new(base_url: Url, auth_token: Option<String>) -> Self {
        Self {
            client: reqwest::ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(3)) //超时
                .build()
                .unwrap(), //没做错误处理，可能会panic

            base_url,
            auth_token,
            include_usage: false, //仿照go实现默认false
        }
    }
}

#[derive(serde::Deserialize)]
struct FunctionStatus {
    replicas: u64,
    available_replicas: u64,
    labels: Option<std::collections::HashMap<String, String>>,
    annotations: std::collections::HashMap<String, String>,
}

#[async_trait]
impl ServiceQuery for ExternalServiceQuery {
    //对ExternalServiceQuery实现ServiceQuery
    async fn get_replicas(
        &self,
        service: &str,
        namespace: &str,
    ) -> Result<ServiceQueryResponse, ScalingError> {
        // 补全URL参数
        let url = format!(
            "{}system/function/{}?namespace={}&usage={}",
            self.base_url, service, namespace, self.include_usage
        );
        //Url样例：http://gateway:8080/system/function/user-service?namespace=dev&usage=true

        let mut req = self.client.get(&url); //向url发起get请求
        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token); //auth
        }

        let resp = req.send().await.map_err(|e| {
            ScalingError::HttpError(
                StatusCode::INTERNAL_SERVER_ERROR, //发送send请求，捕捉返回可能的500状态码
                format!("Request failed: {}", e),
            )
        })?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| {
            ScalingError::HttpError(status, format!("Failed to read response: {}", e))
        })?;

        match status {
            StatusCode::OK => {
                let function: FunctionStatus = serde_json::from_str(&body)?;//http响应体字符body反序列化为FunctionStatus
                parse_function_status(function)//标签解析
            }
            _ => Err(ScalingError::HttpError(status, body)),
        }
    }

    async fn set_replicas(
        &self,
        service: &str,
        namespace: &str,
        count: u64,
    ) -> Result<(), ScalingError> {
        let url = format!(
            "{}system/scale-function/{}?namespace={}",
            self.base_url, service, namespace
        );

        let payload = serde_json::json!({
            "serviceName": service,
            "replicas": count
        });

        let mut req = self.client.post(&url).json(&payload);

        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }

        let start = std::time::Instant::now();
        let resp = req.send().await.map_err(|e| {
            ScalingError::HttpError(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Request failed: {}", e),
            )
        })?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| {
            ScalingError::HttpError(status, format!("Failed to read response: {}", e))
        })?;

        log::info!(
            "SetReplicas [{}.{}] took: {:.4}s",
            service,
            namespace,
            start.elapsed().as_secs_f64()
        );

        match status {
            StatusCode::OK | StatusCode::ACCEPTED => Ok(()),
            _ => Err(ScalingError::HttpError(
                status,
                format!("{} - {} - {} - {}", body, url, service, namespace),
            )),
        }
    }
}

fn parse_function_status(function: FunctionStatus) -> Result<ServiceQueryResponse, ScalingError> {
    let mut min = DEFAULT_MIN_REPLICAS;
    let mut max = DEFAULT_MAX_REPLICAS;
    let mut factor = DEFAULT_SCALING_FACTOR;

    if let Some(labels) = &function.labels {
        min = parse_label(labels, "min_scale", min)?;
        max = parse_label(labels, "max_scale", max)?;
        factor = parse_label(labels, "scaling_factor", factor)?;

        if factor == 0 || factor > 100 {
            return Err(ScalingError::InvalidFactor(factor as u64));
        }
    }

    Ok(ServiceQueryResponse {
        replicas: function.replicas,
        max_replicas: max,
        min_replicas: min,
        scaling_factor: factor,
        available_replicas: function.available_replicas,
        annotations: function.annotations.clone(),
    })
}

// 标签解析工具函数
fn parse_label<T: std::str::FromStr>(
    labels: &HashMap<String, String>,
    key: &str,
    default: T,
) -> Result<T, ScalingError> {
    labels
        .get(key)
        .map(|v| {
            v.parse()
                .map_err(|_| ScalingError::LabelParse(format!("Invalid {} value: {}", key, v)))
        })
        .transpose()
        .map(|v| v.unwrap_or(default))
}
pub fn calculate_replicas(
    status: &str,
    current: u64,
    min: u64,
    user_max: u64,
    scaling_factor: u8,
) -> u64 {
    // 计算实际最大副本数（取用户设置和系统默认的较小值）
    let max_replicas = std::cmp::min(user_max, DEFAULT_MAX_REPLICAS);
    // 计算步长：最大副本数的百分比，向上取整
    let step = ((max_replicas as f64 / 100.0) * scaling_factor as f64).ceil() as u64;

    match (status, step) {
        ("firing", 0) => current, // 步长为0时保持现状
        ("firing", _) => {
            // 计算增量扩容后的副本数，不超过最大值
            let proposed = current.saturating_add(step);//表示安全相加，防止溢出问题
            std::cmp::min(proposed, max_replicas)
        }
        _ => {
            // 非触发状态时重置为最小副本数，但不得低于当前值
            max(current, min)
        }
    }
}

