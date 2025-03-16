use super::{function_cache::FunctionCache, scaling_config::ScalingConfig, service_query::ServiceQuery};
use std::time;
use super::scaling_error::ScalingError;
pub struct FunctionScaler {
    pub cache: FunctionCache,
    pub config: ScalingConfig,
    //SingleFlight: 用来将多个并发请求合并为一，未实现
}
pub struct FunctionScaleResult{
    available: bool,
    error:     ScalingError,
	found:     bool,
	duration:  time::Duration
}
impl FunctionScaler {
    fn new(config: ScalingConfig,functiongCacher: FunctionCache)->FunctionScaler{
        return FunctionScaler {
            cache: functiongCacher,
            config: config
        }

    }
    async fn Scale(&self,functionName: &str,namespace: &str)->FunctionScaleResult{
        let start = time::Instant::now();
        let (cachedResponse,hit)=self.cache.get(functionName,namespace).await;
        if cachedResponse.available_replicas>0 && hit{
            return FunctionScaleResult {
                available: true,
                error : ScalingError::None,
                found : true,
                duration: start.elapsed(),
            }
        }
        let res = self.config.ServiceQuery.get_replicas(functionName, namespace);

        FunctionScaleResult {
            available: true,
            error: ScalingError::None,
            found : true,
            duration: start.elapsed(),

        }
    }
} 
