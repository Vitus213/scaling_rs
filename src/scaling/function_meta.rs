use std::time::{Duration, SystemTime};
use super::service_query::ServiceQueryResponse;
// FunctionMeta holds the last refresh time and any other
// metadata needed for caching.
pub struct FunctionMeta {
    last_refresh: SystemTime,
    service_query_response: ServiceQueryResponse,
}

// ServiceQueryResponse is a placeholder for the actual response structure.
// You should replace this with the actual definition.

impl FunctionMeta {
    // Create a new FunctionMeta instance
    pub fn new(service_query_response: ServiceQueryResponse) -> Self {
        Self {
            last_refresh: SystemTime::now(),
            service_query_response,
        }
    }

    // Check if the cache item has expired given the expiry duration
    pub fn is_expired(&self, expiry: Duration) -> bool {
        match self.last_refresh.elapsed() {
            Ok(elapsed) => elapsed > expiry,
            Err(_) => true, // If time went backward, consider it expired
        }
    }
}