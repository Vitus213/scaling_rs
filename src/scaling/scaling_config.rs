use std::time;
use super::service_query::ExternalServiceQuery;
pub struct ScalingConfig {
    pub MaxPOllcount: u64,
    pub FunctionPollInterva: time::Duration,
    pub CacheExpiry: time::Duration,
    pub ServiceQuery: ExternalServiceQuery,
    pub SetScaleRetrie: u64
}