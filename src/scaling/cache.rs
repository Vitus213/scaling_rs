//缓存层实现
// cache.rs
use dashmap::DashMap;
use std::time::{Duration, Instant};

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
    
    pub async fn get_or_update(
        &self,
        function: &str,
        namespace: &str,
        service_query: &dyn ServiceQuery,
    ) -> Result<ServiceQueryResponse, ScalingError> {
        let key = format!("{}:{}", namespace, function);
        
        if let Some(entry) = self.cache.get(&key) {
            if entry.1.elapsed() < self.ttl {
                return Ok(entry.0.clone());
            }
        }
        
        let resp = service_query.get_replicas(function, namespace).await?;
        self.cache.insert(key, (resp.clone(), Instant::now()));
        Ok(resp)
    }
}