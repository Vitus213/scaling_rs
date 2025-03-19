
use super::service_query::ServiceQueryResponse;
use super::{function_cache::FunctionCache, scaling_config::ScalingConfig, service_query::ServiceQuery};
use std::time;
use super::scaling_error::ScalingError;

use tokio::time::sleep;

pub struct FunctionScaler {
    pub Cache: FunctionCache,
    pub Config: ScalingConfig,
    //SingleFlight: 用来将多个并发请求合并为一，未实现
}
pub struct FunctionScaleResult{
    Available: bool,
    Error:     Result<(),ScalingError>,
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
                Error : Ok(()),
                Found : true,
                Duration: start.elapsed(),
            }
        }
        // The wasn't a hit, or there were no available replicas found
        let result = self.Config.ServiceQuery.get_replicas(functionName, namespace).await;
        let res;
        match result {
            Ok(v) =>   res =v,
            Err(err) => {
                return FunctionScaleResult{
                    Error:     Err((err)),
                    Available: false,
                    Found:     false,
                    Duration:  start.elapsed(),
                }
            },
        };
        if res.available_replicas > 0{
            return FunctionScaleResult{
                Error:     Ok(()),
                Available: true,
                Found:     true,
                Duration:  start.elapsed(),
            }
        }
        self.Cache.set(functionName,namespace,&res).await;
        if res.replicas == 0 {
            let mut minReplicas :u64 = 1;
            if res.min_replicas>0{
                minReplicas = res.min_replicas
            }
            // In a retry-loop, first query desired replicas, then
		    // set them if the value is still at 0.
            let mut scale_result = Ok(());
            let mut routine = |attempt: u64|async move {
                let result = self.Config.ServiceQuery.get_replicas(functionName, namespace).await;
                let res ;
                match result{
                     Err(err) => return Err(err),
                    Ok(v) =>  res =v,
                };
                self.Cache.set(functionName,namespace,&res).await;
                if res.available_replicas > 0{
                    return Ok(());
                }
                let setResult = self.Config.ServiceQuery.set_replicas(functionName, namespace, minReplicas).await;
                println!("{}", format!("[Scale {}/{}] function={} 0 => {} requested",//log print to be done
                attempt, self.Config.SetScaleRetrie, functionName, minReplicas));
                match setResult {
                    Ok(())=> return Ok(()),
                    Err(err)=> {
                        println!("{}",format!("unable to scale function {}, err: {}", functionName, err));
                         return Err(err)
                    },
                }
            };
            for i in 1..self.Config.SetScaleRetrie{
                let res= routine(i).await;
                match res{
                    Ok(())=>{
                        break
                    },
                    Err(err)=>{
                        println!("Scale fail:{}/{},error:{}",i,self.Config.SetScaleRetrie,err);
                        let scale_result:Result<(), ScalingError> = Err(err);
                    }
                }
                sleep(self.Config.FunctionPollInterva).await;
            }
            match scale_result{
                Ok(())=>{

                }
                Err(err)=>{
                    return FunctionScaleResult {
                        Available: false,
                        Error : Err(err),
                        Found: true,
                        Duration: start.elapsed(),
                    }
                }
            }
        } 
        // Holding pattern for at least one function replica to be available
        for i in 1..self.Config.MaxPOllcount {
            let result = self.Config.ServiceQuery.get_replicas(functionName, namespace).await;
            let mut query_response:ServiceQueryResponse;
            match result {
                Ok(v)=>{
                    query_response = v;
                }
                Err(err)=>{
                    return FunctionScaleResult {
                        Available: false,
                        Error : Err(err),
                        Found: true,
                        Duration: start.elapsed(),
                    }
                }
            }
            self.Cache.set(functionName,namespace,&query_response).await;
            if query_response.available_replicas > 0{
                //log print to be done
                return FunctionScaleResult {
                    Available: true,
                    Error : Ok(()),
                    Found: true,
                    Duration: start.elapsed(),
                }
            }
            sleep(self.Config.FunctionPollInterva).await;
        }       
        // Holding pattern for at least one function replica to be available
        for i in 1..self.Config.MaxPOllcount {
            let result = self.Config.ServiceQuery.get_replicas(functionName, namespace).await;
            let mut query_response:ServiceQueryResponse;
            match result {
                Ok(v)=>{
                    query_response = v;
                }
                Err(err)=>{
                    return FunctionScaleResult {
                        Available: false,
                        Error : Err(err),
                        Found: true,
                        Duration: start.elapsed(),
                    }
                }
            }
            self.Cache.set(functionName,namespace,&query_response).await;
            if query_response.available_replicas > 0{
                //log print to be done
                return FunctionScaleResult {
                    Available: true,
                    Error : Ok(()),
                    Found: true,
                    Duration: start.elapsed(),
                }
            }
            sleep(self.Config.FunctionPollInterva).await;
        }
        return FunctionScaleResult {
            Available: true,
            Error : Ok(()),
            Found: true,
            Duration: start.elapsed(),
        }
    }
}

