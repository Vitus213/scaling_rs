//FunctionScaler: 用于从零缩扩容函数
//scale为makescalehandler的一个子函数，用于缩放函数
//FunctionScaleResult: 用于返回缩放结果
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
        // The wasn't a hit, or there were no available replicas found
        let result = self.Config.ServiceQuery.get_replicas(functionName, namespace).await;
        let res;
        match result {
            Ok(v) =>   res =v,
            Err(err) => {
                return FunctionScaleResult{
                    Error:     err,
                    Available: false,
                    Found:     false,
                    Duration:  start.elapsed(),
                }
            },
        };
        if res.available_replicas > 0{
            return FunctionScaleResult{
                Error:     ScalingError::None,
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
            let mut scale_result = ScalingError::None;
            let mut routine = |attempt: u64|async move {
                let result = self.Config.ServiceQuery.get_replicas(functionName, namespace).await;
                let res ;
                match result{
                     Err(err) => return err,
                    Ok(v) =>  res =v,
                };
                self.Cache.set(functionName,namespace,&res).await;
                if res.available_replicas > 0{
                    return ScalingError::None;
                }
                let setResult = self.Config.ServiceQuery.set_replicas(functionName, namespace, minReplicas).await;
                println!("{}", format!("[Scale {}/{}] function={} 0 => {} requested",//log print to be done
                attempt, self.Config.SetScaleRetrie, functionName, minReplicas));
                match setResult {
                    Ok(())=> return ScalingError::None,
                    Err(err)=> {
                        println!("{}",format!("unable to scale function {}, err: {}", functionName, err));
                         return err
                    },
                }
            };
            for i in 1..self.Config.SetScaleRetrie{
                let res= routine(i).await;
                match res{
                    ScalingError::None=>{
                        break
                    },
                    ScalingError::HttpError(a,b )=>{
                        println!("Scale fail:{}/{},error:httperror{},{}",i,self.Config.SetScaleRetrie,a,b);
                        let scale_result = ScalingError::HttpError((a), (b));
                    }
                    ScalingError::InvalidFactor(a)=>{
                        println!("Scale fail:{}/{},error:invalid factor{}",i,self.Config.SetScaleRetrie,a);
                        let scale_result = ScalingError::InvalidFactor((a));
                    }
                    ScalingError::JsonError(a)=>{
                        println!("Scale fail:{}/{},error:json error{}",i,self.Config.SetScaleRetrie,a);
                        let scale_result = ScalingError::JsonError((a));
                    }
                    ScalingError::LabelParse(a)=>{
                        println!("Scale fail:{}/{},error:label parse{}",i,self.Config.SetScaleRetrie,a);
                        let scale_result = ScalingError::LabelParse((a));
                    }
                }
                sleep(self.Config.FunctionPollInterva).await;
            }
            match scale_result{
                ScalingError::None=>{

                }
                ScalingError::HttpError(a,b )=>{
                    return FunctionScaleResult {
                        Available: false,
                        Error : ScalingError::HttpError(a,b),
                        Found: true,
                        Duration: start.elapsed(),
                    }
                }
                ScalingError::InvalidFactor(a)=>{
                    return FunctionScaleResult {
                        Available: false,
                        Error : ScalingError::InvalidFactor(a),
                        Found: true,
                        Duration: start.elapsed(),
                    }
                }
                ScalingError::JsonError(a)=>{
                    return FunctionScaleResult {
                        Available: false,
                        Error : ScalingError::JsonError(a),
                        Found: true,
                        Duration: start.elapsed(),
                    }
                }
                ScalingError::LabelParse(a)=>{
                    return FunctionScaleResult {
                        Available: false,
                        Error : ScalingError::LabelParse(a),
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
                        Error : err,
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
                    Error : ScalingError::None,
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
                        Error : err,
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
                    Error : ScalingError::None,
                    Found: true,
                    Duration: start.elapsed(),
                }
            }
            sleep(self.Config.FunctionPollInterva).await;
        }
        return FunctionScaleResult {
            Available: true,
            Error : ScalingError::None,
            Found: true,
            Duration: start.elapsed(),
        }
    }
}

