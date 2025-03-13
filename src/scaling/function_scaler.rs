use super::{function_cacher::FunctionCache, scaling_config::ScalingConfig, service_query::ServiceQuery};
use std::time;
use super::scaling_error::ScalingError;
pub struct FunctionScaler {
    pub Cache: FunctionCache,
    pub Config: ScalingConfig,
    //SingleFlight: 用来将多个并发请求合并为一，未实现
}
pub struct FunctionScaleResult{
    Available: bool,
    Error:     ScalingError,
	Found:     bool,
	Duration:  time::Duration
}
impl FunctionScaler {
    fn new(config: ScalingConfig,functiongCacher: FunctionCache)->FunctionScaler{
        return FunctionScaler {
            Cache: functiongCacher,
            Config: config
        }

    }
    async fn Scale(&self,functionName: &str,namespace: &str)->FunctionScaleResult{
        let start = time::Instant::now();
        let (cachedResponse,hit)=self.Cache.get(functionName,namespace).await;
        if cachedResponse.available_replicas>0 && hit{
            return FunctionScaleResult {
                Available: true,
                Error : ScalingError::None,
                Found : true,
                Duration: start.elapsed(),
            }
        }
        let res = self.Config.ServiceQuery.get_replicas(functionName, namespace);

        FunctionScaleResult {
            Available: true,
            Error: ScalingError::None,
            Found : true,
            Duration: start.elapsed(),

        }
    }
} 
