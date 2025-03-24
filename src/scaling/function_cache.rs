//缓存层实现
// cache.rs
use dashmap::DashMap;
use std::time::{Duration, Instant};
use super::{service_query::ServiceQueryResponse,scaling_error::ScalingError};
use super::service_query::ServiceQuery;
use std::collections::HashMap;
use tokio::time::{sleep,};

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
    
    pub async fn get(
        &self,
        function: &str,
        namespace: &str,
    ) -> (ServiceQueryResponse,bool) {
        let key = format!("{}.{}", function, namespace);
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
    pub async fn set(&self,function: &str, namespace: &str, queryRes: &ServiceQueryResponse){
        let key = format!("{}.{}", function, namespace);
        self.cache.insert(key,(queryRes.clone(),Instant::now()));
    }
}
#[cfg(test)]
mod tests {
    use actix_web::rt::time;

    use super::*;

    #[actix_rt::test]
    async fn test_CacheExpiresIn1MS(){
        let before = time::Instant::now();
        let fnName = "echo";
        let namespace = "";
        let cache = FunctionCache{
            cache: DashMap::new(),
            ttl: Duration::from_millis(1)
        };
        let sqr = ServiceQueryResponse {
            replicas: 1,
            max_replicas: 2,
            min_replicas: 1,
            scaling_factor: 1,
            available_replicas: 1,
            annotations: HashMap::new(),
        };
        cache.set(fnName, namespace, &sqr).await;
        time::sleep(Duration::from_millis(2)).await;
        let (a,b)=cache.get(fnName, namespace).await;
        assert_eq!(b,false);
    }
    #[actix_rt::test]
    async fn Test_CacheGivesHitWithLongExpiry () {
        let fnName = "echo";
        let namespace = "";
        let cache = FunctionCache{
            cache: DashMap::new(),
            ttl: Duration::from_millis(500)
        };
        let sqr = ServiceQueryResponse {
            replicas: 1,
            max_replicas: 2,
            min_replicas: 1,
            scaling_factor: 1,
            available_replicas: 1,
            annotations: HashMap::new(),
        };
        cache.set(fnName, namespace, &sqr).await;
        let (a,b)=cache.get(fnName, namespace).await;
        assert_eq!(b,true);

    }
    #[actix_rt::test]
    async fn Test_CacheFunctionNotExist (){
        let fnName = "echo";
        let namespace = "";
        let testName = "burt";
        let cache = FunctionCache{
            cache: DashMap::new(),
            ttl: Duration::from_millis(100)
        };
        let sqr = ServiceQueryResponse {
            replicas: 1,
            max_replicas: 2,
            min_replicas: 1,
            scaling_factor: 1,
            available_replicas: 1,
            annotations: HashMap::new(),
        };
        cache.set(fnName, namespace, &sqr);
        time::sleep(Duration::from_millis(2)).await;
        let (a,b)=cache.get(testName, namespace).await;
        assert_eq!(b,false);
    }
}

