
use reqwest::StatusCode;
use serde_json::Error as JsonError;
#[derive(thiserror::Error,Debug)]
//要重构ScalingError，细化错误类型
pub enum ScalingError {
    #[error("HTTP error ({0}): {1}")]
    HttpError(StatusCode, String),
    
    #[error("JSON parse error: {0}")]
    JsonError(#[from] JsonError),
    
    #[error("Invalid scaling factor {0} (must be 1-100)")]
    InvalidFactor(u64),
    
    #[error("Label parse error: {0}")]
    LabelParse(String), 

    #[error("No error")]
    None, // 添加一个表示“无错误”的变体
}
//如果后面添加了错误类型需要去 scale函数里的几个枚举把分支加上

