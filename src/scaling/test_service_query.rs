#[cfg(test)]
mod tests {
    use crate::scaling::service_query::ServiceQuery;
    use crate::scaling::service_query::calculate_replicas;
    use mockito::{mock, Matcher};
    use reqwest::Url;
    use crate::scaling::service_query::ExternalServiceQuery;
    #[test]
    fn test_firing_scaling() {
        // 触发扩容：当前2 + 步长3 = 5
        assert_eq!(calculate_replicas("firing", 2, 1, 10, 30), 5);
    }
    #[test]
    fn test_resolved_status() {
        // 状态解除后重置为最小值（但不得低于当前值）
        assert_eq!(calculate_replicas("resolved", 3, 5, 10, 30), 5);
        assert_eq!(calculate_replicas("resolved", 8, 5, 10, 30), 8);
    }

    #[test]
    fn test_zero_step() {
        // 缩放因子为0时保持现状
        assert_eq!(calculate_replicas("firing", 5, 1, 10, 0), 5);
    }


    #[tokio::test]
    async fn test_get_replicas() {
        // 模拟服务端返回的响应体
        let _mock = mock("GET", "/system/function/test-service")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("namespace".into(), "test-namespace".into()),
                Matcher::UrlEncoded("usage".into(), "false".into()),
            ]))
            .with_status(200)
            .with_body(r#"
                {
                    "replicas": 3,
                    "available_replicas": 3,
                    "labels": {
                        "min_scale": "1",
                        "max_scale": "10",
                        "scaling_factor": "20"
                    },
                    "annotations": {}
                }
            "#)
            .create();

        // 创建 ExternalServiceQuery 实例
        let base_url = Url::parse(&mockito::server_url()).unwrap();
        let service_query = ExternalServiceQuery::new(base_url, None);

        // 调用 get_replicas 方法
        let response = service_query
            .get_replicas("test-service", "test-namespace")
            .await
            .unwrap();

        // 验证返回值
        assert_eq!(response.replicas, 3);
        assert_eq!(response.min_replicas, 1);
        assert_eq!(response.max_replicas, 10);
        assert_eq!(response.scaling_factor, 20);
    }

    #[tokio::test]
    async fn test_set_replicas() {
        // 模拟服务端返回的响应体
        let _mock = mock("POST", "/system/scale-function/test-service")
            .match_query(Matcher::UrlEncoded("namespace".into(), "test-namespace".into()))
            .match_body(Matcher::JsonString(
                r#"{"serviceName":"test-service","replicas":5}"#.into(),
            ))
            .with_status(200)
            .create();

        // 创建 ExternalServiceQuery 实例
        let base_url = Url::parse(&mockito::server_url()).unwrap();
        let service_query = ExternalServiceQuery::new(base_url, None);

        // 调用 set_replicas 方法
        let result = service_query
            .set_replicas("test-service", "test-namespace", 5)
            .await;

        // 验证返回值
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_get_replicas_500(){
            let _mock = mock("GET", "/system/function/test-service")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("namespace".into(), "test-namespace".into()),
                Matcher::UrlEncoded("usage".into(), "false".into()),
            ]))
            .with_status(200)
            .with_body(r#"
                {
                    "replicas": 3,
                    "available_replicas": 3,
                    "labels": {
                        "min_scale": "1",
                        "max_scale": "10",
                        "scaling_factor": "20"
                    },
                    "annotations": {}
                }
            "#)
            .create();

        // 创建 ExternalServiceQuery 实例
        let base_url = Url::parse(&mockito::server_url()).unwrap();
        let service_query = ExternalServiceQuery::new(base_url, None);

        // 调用 get_replicas 方法
        let response = service_query
            .get_replicas("test-service", "test-namespace")
            .await
            .unwrap();

        // 验证返回值
        assert_eq!(response.replicas, 3);
        assert_eq!(response.min_replicas, 1);
        assert_eq!(response.max_replicas, 10);
        assert_eq!(response.scaling_factor, 20);
    }
}
