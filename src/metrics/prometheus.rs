//Prometheus结构体实现
use serde::{Deserialize, Serialize};
// 对应 PrometheusInnerAlertLabel
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusInnerAlertLabel {
    #[serde(rename = "alertname")]
    alert_name: String,
    
    #[serde(rename = "function_name")]
    pub function_name: String,
}

// 对应 PrometheusInnerAlert
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusInnerAlert {//不知道这里要不要json重命名
    status: String,
    pub labels: PrometheusInnerAlertLabel,
}
impl PrometheusInnerAlertLabel{
    pub fn get_function_name(&self) -> Option<String> {
        Some(self.function_name.clone())
    }
}

// 对应 PrometheusAlert
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusAlert {
    status: String,
    receiver: String,
    pub alerts: Vec<PrometheusInnerAlert>,
}