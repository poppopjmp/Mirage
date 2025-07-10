use crate::models::ConfigValueType;
use jsonschema::{JSONSchema, ValidationError};
use mirage_common::{Error, Result};
use serde_json::Value;
use std::sync::Arc;

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate_against_schema(&self, value: &Value, schema: &Value) -> Result<()> {
        // Parse and compile schema
        let compiled_schema = JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft7)
            .compile(schema)
            .map_err(|e| Error::Validation(format!("Invalid JSON schema: {}", e)))?;

        // Validate value against schema
        match compiled_schema.validate(value) {
            Ok(_) => Ok(()),
            Err(errors) => {
                let error_messages: Vec<String> = errors.map(|err| format!("{}", err)).collect();

                Err(Error::Validation(format!(
                    "Value does not match schema: {}",
                    error_messages.join(", ")
                )))
            }
        }
    }

    pub fn validate_value_type(&self, value: &Value, value_type: &ConfigValueType) -> Result<()> {
        match value_type {
            ConfigValueType::String => {
                if !value.is_string() {
                    return Err(Error::Validation("Value must be a string".into()));
                }
            }
            ConfigValueType::Integer => {
                if !value.is_i64() && !value.is_u64() {
                    return Err(Error::Validation("Value must be an integer".into()));
                }
            }
            ConfigValueType::Float => {
                if !value.is_f64() && !value.is_i64() && !value.is_u64() {
                    return Err(Error::Validation("Value must be a number".into()));
                }
            }
            ConfigValueType::Boolean => {
                if !value.is_boolean() {
                    return Err(Error::Validation("Value must be a boolean".into()));
                }
            }
            ConfigValueType::Json => {
                if !value.is_object() {
                    return Err(Error::Validation("Value must be a JSON object".into()));
                }
            }
            ConfigValueType::List => {
                if !value.is_array() {
                    return Err(Error::Validation("Value must be an array".into()));
                }
            }
        }

        Ok(())
    }
}
