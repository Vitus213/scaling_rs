use std::time;
use super::service_query::ExternalServiceQuery;
pub struct ScalingConfig {
    pub MaxPOllcount: u64,//最大轮询
    pub FunctionPollInterva: time::Duration,
    pub CacheExpiry: time::Duration,
    pub ServiceQuery: ExternalServiceQuery, //准备好的ServiceQuery
    pub SetScaleRetrie: u64//设置缩放重试次数
}