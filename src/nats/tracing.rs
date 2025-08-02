// Copyright 2025 Cowboy AI, LLC.

//! Distributed tracing support for NATS operations
//!
//! This module provides OpenTelemetry-compatible distributed tracing
//! for commands, events, and queries flowing through NATS.

use async_nats::HeaderMap;
use opentelemetry::{
    global,
    propagation::{Extractor, Injector, TextMapPropagator},
    trace::{
        FutureExt, Span, SpanContext, SpanKind, Status, TraceContextExt, 
        TraceId, Tracer, TracerProvider,
    },
    Context, KeyValue,
};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::error::{NatsError, Result};

/// Trace context that flows with messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Trace ID (128-bit)
    pub trace_id: String,
    
    /// Span ID (64-bit)
    pub span_id: String,
    
    /// Parent span ID
    pub parent_span_id: Option<String>,
    
    /// Trace flags
    pub trace_flags: u8,
    
    /// Baggage items
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    /// Create from OpenTelemetry context
    pub fn from_context(context: &Context) -> Option<Self> {
        let span = context.span();
        let span_context = span.span_context();
        
        if span_context.is_valid() {
            Some(Self {
                trace_id: format!("{:032x}", span_context.trace_id()),
                span_id: format!("{:016x}", span_context.span_id()),
                parent_span_id: None, // Set by propagator
                trace_flags: span_context.trace_flags().to_u8(),
                baggage: HashMap::new(),
            })
        } else {
            None
        }
    }
    
    /// Convert to OpenTelemetry SpanContext
    pub fn to_span_context(&self) -> Option<SpanContext> {
        let trace_id = TraceId::from_hex(&self.trace_id).ok()?;
        let span_id = opentelemetry::trace::SpanId::from_hex(&self.span_id).ok()?;
        
        Some(SpanContext::new(
            trace_id,
            span_id,
            opentelemetry::trace::TraceFlags::new(self.trace_flags),
            false,
            opentelemetry::trace::TraceState::default(),
        ))
    }
}

/// NATS header extractor for OpenTelemetry
struct NatsHeaderExtractor<'a> {
    headers: &'a HeaderMap,
}

impl<'a> Extractor for NatsHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.as_str())
    }
    
    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|k| k.as_str()).collect()
    }
}

/// NATS header injector for OpenTelemetry
struct NatsHeaderInjector<'a> {
    headers: &'a mut HeaderMap,
}

impl<'a> Injector for NatsHeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.headers.insert(key, value);
    }
}

/// Distributed tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name
    pub service_name: String,
    
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: Option<String>,
    
    /// Sampling ratio (0.0 to 1.0)
    pub sampling_ratio: f64,
    
    /// Enable console exporter for debugging
    pub console_debug: bool,
    
    /// Additional resource attributes
    pub resource_attributes: HashMap<String, String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "git-domain".to_string(),
            otlp_endpoint: None,
            sampling_ratio: 1.0,
            console_debug: false,
            resource_attributes: HashMap::new(),
        }
    }
}

/// Distributed tracing manager
pub struct TracingManager {
    tracer: global::BoxedTracer,
    propagator: TraceContextPropagator,
    config: TracingConfig,
}

impl TracingManager {
    /// Initialize tracing with the given configuration
    pub fn init(config: TracingConfig) -> Result<Self> {
        // Build resource
        let mut attributes = vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ];
        
        for (key, value) in &config.resource_attributes {
            attributes.push(KeyValue::new(key.clone(), value.clone()));
        }
        
        let resource = Resource::new(attributes);
        
        // Configure trace provider
        let mut provider_builder = trace::TracerProvider::builder()
            .with_id_generator(RandomIdGenerator::default())
            .with_sampler(Sampler::TraceIdRatioBased(config.sampling_ratio))
            .with_resource(resource);
        
        // Add OTLP exporter if configured
        if let Some(endpoint) = &config.otlp_endpoint {
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
                .with_protocol(Protocol::Grpc);
            
            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .install_batch(opentelemetry_sdk::runtime::Tokio)
                .map_err(|e| NatsError::Other(format!("Failed to install OTLP tracer: {}", e)))?;
                
            provider_builder = provider_builder.with_batch_exporter(
                tracer.provider().unwrap().span_processors()[0].clone(),
                opentelemetry_sdk::runtime::Tokio,
            );
        }
        
        // Add console exporter if debugging
        if config.console_debug {
            let exporter = opentelemetry_stdout::SpanExporter::default();
            provider_builder = provider_builder.with_simple_exporter(exporter);
        }
        
        // Build and set global provider
        let provider = provider_builder.build();
        let tracer = provider.tracer("git-domain");
        global::set_tracer_provider(provider);
        
        // Set up propagator
        let propagator = TraceContextPropagator::new();
        global::set_text_map_propagator(propagator.clone());
        
        info!("Initialized distributed tracing for service: {}", config.service_name);
        
        Ok(Self {
            tracer,
            propagator,
            config,
        })
    }
    
    /// Extract trace context from NATS headers
    pub fn extract_context(&self, headers: &HeaderMap) -> Context {
        let extractor = NatsHeaderExtractor { headers };
        self.propagator.extract(&extractor)
    }
    
    /// Inject trace context into NATS headers
    pub fn inject_context(&self, context: &Context, headers: &mut HeaderMap) {
        let mut injector = NatsHeaderInjector { headers };
        self.propagator.inject_context(context, &mut injector);
        
        // Add custom trace headers
        if let Some(trace_ctx) = TraceContext::from_context(context) {
            headers.insert("X-Trace-ID", trace_ctx.trace_id.clone());
            headers.insert("X-Span-ID", trace_ctx.span_id.clone());
        }
    }
    
    /// Create a span for command processing
    pub fn command_span(&self, command_type: &str, command_id: Uuid) -> global::BoxedSpan {
        self.tracer
            .span_builder(format!("command.{}", command_type))
            .with_kind(SpanKind::Server)
            .with_attributes(vec![
                KeyValue::new("command.type", command_type.to_string()),
                KeyValue::new("command.id", command_id.to_string()),
                KeyValue::new("messaging.system", "nats"),
                KeyValue::new("messaging.destination", format!("git.cmd.{}", command_type)),
            ])
            .start(&self.tracer)
    }
    
    /// Create a span for event publishing
    pub fn event_span(&self, event_type: &str, event_id: Uuid) -> global::BoxedSpan {
        self.tracer
            .span_builder(format!("event.{}", event_type))
            .with_kind(SpanKind::Producer)
            .with_attributes(vec![
                KeyValue::new("event.type", event_type.to_string()),
                KeyValue::new("event.id", event_id.to_string()),
                KeyValue::new("messaging.system", "nats"),
                KeyValue::new("messaging.destination", format!("git.event.{}", event_type)),
            ])
            .start(&self.tracer)
    }
    
    /// Create a span for query processing
    pub fn query_span(&self, query_type: &str) -> global::BoxedSpan {
        self.tracer
            .span_builder(format!("query.{}", query_type))
            .with_kind(SpanKind::Server)
            .with_attributes(vec![
                KeyValue::new("query.type", query_type.to_string()),
                KeyValue::new("messaging.system", "nats"),
                KeyValue::new("messaging.destination", format!("git.query.{}", query_type)),
            ])
            .start(&self.tracer)
    }
    
    /// Create a span for projection updates
    pub fn projection_span(&self, projection_name: &str, event_type: &str) -> global::BoxedSpan {
        self.tracer
            .span_builder(format!("projection.{}.{}", projection_name, event_type))
            .with_kind(SpanKind::Consumer)
            .with_attributes(vec![
                KeyValue::new("projection.name", projection_name.to_string()),
                KeyValue::new("event.type", event_type.to_string()),
                KeyValue::new("messaging.system", "nats"),
            ])
            .start(&self.tracer)
    }
    
    /// Record an error on the current span
    pub fn record_error(&self, span: &global::BoxedSpan, error: &dyn std::error::Error) {
        span.record_error(error);
        span.set_status(Status::error(error.to_string()));
    }
    
    /// Add event to the current span
    pub fn add_event(&self, span: &global::BoxedSpan, name: &str, attributes: Vec<KeyValue>) {
        span.add_event(name, attributes);
    }
}

/// Extension trait for adding tracing to NATS operations
pub trait TracedNatsExt {
    /// Execute with distributed tracing
    async fn with_tracing<F, T>(self, f: F) -> T
    where
        F: FnOnce() -> T;
}

/// Trace context propagation for commands
pub struct TracedCommand {
    pub command_id: Uuid,
    pub command_type: String,
    pub trace_context: Option<TraceContext>,
}

/// Trace context propagation for events
pub struct TracedEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub correlation_id: Uuid,
    pub causation_id: Uuid,
    pub trace_context: Option<TraceContext>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trace_context_serialization() {
        let ctx = TraceContext {
            trace_id: "0123456789abcdef0123456789abcdef".to_string(),
            span_id: "0123456789abcdef".to_string(),
            parent_span_id: Some("fedcba9876543210".to_string()),
            trace_flags: 1,
            baggage: HashMap::new(),
        };
        
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: TraceContext = serde_json::from_str(&json).unwrap();
        
        assert_eq!(ctx.trace_id, deserialized.trace_id);
        assert_eq!(ctx.span_id, deserialized.span_id);
        assert_eq!(ctx.parent_span_id, deserialized.parent_span_id);
        assert_eq!(ctx.trace_flags, deserialized.trace_flags);
    }
    
    #[test]
    fn test_header_injection() {
        let mut headers = HeaderMap::new();
        headers.insert("existing", "value");
        
        let mut injector = NatsHeaderInjector { headers: &mut headers };
        injector.set("traceparent", "00-trace-span-01".to_string());
        
        assert_eq!(headers.get("traceparent").unwrap().as_str(), "00-trace-span-01");
        assert_eq!(headers.get("existing").unwrap().as_str(), "value");
    }
}