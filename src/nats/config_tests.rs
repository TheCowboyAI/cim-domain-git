// Copyright 2025 Cowboy AI, LLC.

//! Tests for NATS configuration

#[cfg(test)]
#[cfg(feature = "outdated_tests_disabled")]
mod tests {
    use super::super::config::*;
    use std::time::Duration;

    #[test]
    fn test_nats_config_default() {
        let config = NatsConfig::default();
        assert_eq!(config.url, "nats://localhost:4222");
        assert_eq!(config.service.name, "git-domain");
        assert!(config.auth.is_none());
        assert!(config.tls.is_none());
        assert_eq!(config.retry.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.retry.reconnect_delay, Duration::from_secs(5));
        assert_eq!(config.retry.max_reconnects, Some(60));
        assert!(config.jetstream.enabled);
    }

    #[test]
    fn test_nats_config_builder() {
        let config = NatsConfig::builder()
            .url("nats://custom:4222")
            .name("test-service")
            .token("secret-token")
            .enable_tls()
            .connect_timeout(Duration::from_secs(5))
            .subject_prefix("test")
            .build();

        assert_eq!(config.url, "nats://custom:4222");
        assert_eq!(config.name, "test-service");
        assert!(matches!(config.auth, NatsAuth::Token(_)));
        assert!(matches!(config.tls, NatsTls::Enabled { .. }));
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert_eq!(config.subject_prefix, "test");
    }

    #[test]
    fn test_nats_auth_variants() {
        // Test None auth
        let auth = NatsAuth::None;
        assert!(matches!(auth, NatsAuth::None));

        // Test Token auth
        let auth = NatsAuth::Token("my-token".to_string());
        if let NatsAuth::Token(token) = &auth {
            assert_eq!(token, "my-token");
        } else {
            panic!("Expected Token variant");
        }

        // Test UserPassword auth
        let auth = NatsAuth::UserPassword {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        if let NatsAuth::UserPassword { username, password } = &auth {
            assert_eq!(username, "user");
            assert_eq!(password, "pass");
        } else {
            panic!("Expected UserPassword variant");
        }

        // Test NKey auth
        let auth = NatsAuth::NKey("nkey-seed".to_string());
        if let NatsAuth::NKey(seed) = &auth {
            assert_eq!(seed, "nkey-seed");
        } else {
            panic!("Expected NKey variant");
        }

        // Test Jwt auth
        let auth = NatsAuth::Jwt {
            jwt: "jwt-token".to_string(),
            seed: "jwt-seed".to_string(),
        };
        if let NatsAuth::Jwt { jwt, seed } = &auth {
            assert_eq!(jwt, "jwt-token");
            assert_eq!(seed, "jwt-seed");
        } else {
            panic!("Expected Jwt variant");
        }
    }

    #[test]
    fn test_nats_tls_variants() {
        // Test None TLS
        let tls = NatsTls::None;
        assert!(matches!(tls, NatsTls::None));

        // Test Enabled TLS
        let tls = NatsTls::Enabled {
            cert_path: None,
            key_path: None,
            ca_path: None,
        };
        assert!(matches!(tls, NatsTls::Enabled { .. }));

        // Test Enabled TLS with paths
        let tls = NatsTls::Enabled {
            cert_path: Some("/path/to/cert".to_string()),
            key_path: Some("/path/to/key".to_string()),
            ca_path: Some("/path/to/ca".to_string()),
        };
        if let NatsTls::Enabled {
            cert_path,
            key_path,
            ca_path,
        } = &tls
        {
            assert_eq!(cert_path.as_deref(), Some("/path/to/cert"));
            assert_eq!(key_path.as_deref(), Some("/path/to/key"));
            assert_eq!(ca_path.as_deref(), Some("/path/to/ca"));
        } else {
            panic!("Expected Enabled variant");
        }
    }

    #[test]
    fn test_config_builder_all_auth_methods() {
        // Test with user/password
        let config = NatsConfig::builder()
            .url("nats://localhost:4222")
            .user_password("user", "pass")
            .build();
        assert!(matches!(config.auth, NatsAuth::UserPassword { .. }));

        // Test with nkey
        let config = NatsConfig::builder()
            .url("nats://localhost:4222")
            .nkey("nkey-seed")
            .build();
        assert!(matches!(config.auth, NatsAuth::NKey(_)));

        // Test with JWT
        let config = NatsConfig::builder()
            .url("nats://localhost:4222")
            .jwt("jwt-token", "jwt-seed")
            .build();
        assert!(matches!(config.auth, NatsAuth::Jwt { .. }));
    }

    #[test]
    fn test_config_builder_tls_options() {
        // Test basic TLS enable
        let config = NatsConfig::builder()
            .url("nats://localhost:4222")
            .enable_tls()
            .build();
        assert!(matches!(config.tls, NatsTls::Enabled { .. }));

        // Test TLS with certificates
        let config = NatsConfig::builder()
            .url("nats://localhost:4222")
            .tls_cert("/cert", "/key")
            .build();
        if let NatsTls::Enabled {
            cert_path,
            key_path,
            ca_path,
        } = &config.tls
        {
            assert_eq!(cert_path.as_deref(), Some("/cert"));
            assert_eq!(key_path.as_deref(), Some("/key"));
            assert!(ca_path.is_none());
        } else {
            panic!("Expected Enabled TLS");
        }

        // Test TLS with CA
        let config = NatsConfig::builder()
            .url("nats://localhost:4222")
            .tls_ca("/ca")
            .build();
        if let NatsTls::Enabled {
            cert_path,
            key_path,
            ca_path,
        } = &config.tls
        {
            assert!(cert_path.is_none());
            assert!(key_path.is_none());
            assert_eq!(ca_path.as_deref(), Some("/ca"));
        } else {
            panic!("Expected Enabled TLS");
        }
    }

    #[test]
    fn test_config_builder_jetstream_options() {
        // Test disabling JetStream
        let config = NatsConfig::builder()
            .url("nats://localhost:4222")
            .no_jetstream()
            .build();
        assert!(!config.jetstream);

        // Default should have JetStream enabled
        let config = NatsConfig::builder().url("nats://localhost:4222").build();
        assert!(config.jetstream);
    }

    #[test]
    fn test_config_from_env() {
        // Test would require setting environment variables
        // This is a placeholder for environment-based configuration testing
        std::env::set_var("NATS_URL", "nats://env-test:4222");
        std::env::set_var("NATS_TOKEN", "env-token");
        
        // In a real implementation, we'd have a from_env() method
        // let config = NatsConfig::from_env().unwrap();
        // assert_eq!(config.url, "nats://env-test:4222");
        
        // Clean up
        std::env::remove_var("NATS_URL");
        std::env::remove_var("NATS_TOKEN");
    }
}