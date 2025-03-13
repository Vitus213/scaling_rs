//缓存层实现
// cache.rs
use dashmap::DashMap;
use std::time::{Duration, Instant};
use super::{service_query::ServiceQueryResponse,scaling_error::ScalingError};
use super::service_query::ServiceQuery;
use std::collections::HashMap;
pub struct FunctionCache {
    cache: DashMap<String, (ServiceQueryResponse, Instant)>,
    ttl: Duration,
}

impl FunctionCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: DashMap::new(),
            ttl,
        }
    }
    
    pub async  fn get(
        &self,
        function: &str,
        namespace: &str,
    ) -> (ServiceQueryResponse,bool) {
        let key = format!("{}:{}", namespace, function);
        let queryRes  = ServiceQueryResponse {
            replicas: 0,
            min_replicas: 0,
            max_replicas: 0,
            scaling_factor: 0,
            available_replicas:0,
            annotations: HashMap::new(),
        };
        if let Some(entry) = self.cache.get(&key) {
            if entry.1.elapsed() < self.ttl {
                return (entry.0.clone(),true);
            }
            else{
                return (entry.0.clone(),false);
            }
        }
        else{
            return (queryRes,false);
        }
    }
    pub  fn set(){

    }
}