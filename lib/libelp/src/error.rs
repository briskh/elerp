use thiserror::Error;

/// User-facing configuration errors
/// 
/// This module provides user-friendly error types for configuration operations.
/// These errors are designed to be helpful for end users and provide clear
/// guidance on how to resolve configuration issues.
#[derive(Error, Debug)]
pub enum ConfigurationError {
    /// TOML parsing error
    #[error("Failed to parse TOML configuration: {message}")]
    TomlParseError {
        /// Human-readable error message
        message: String,
        /// Line number where the error occurred (if available)
        line: Option<usize>,
        /// Column number where the error occurred (if available)
        column: Option<usize>,
    },

    /// TOML serialization error
    #[error("Failed to serialize configuration to TOML: {message}")]
    TomlSerializeError {
        /// Human-readable error message
        message: String,
    },

    /// Configuration validation error
    #[error("Configuration validation failed: {message}")]
    ValidationError {
        /// Human-readable error message
        message: String,
        /// Field name that failed validation (if applicable)
        field: Option<String>,
    },

    /// File I/O error
    #[error("File operation failed: {message}")]
    FileError {
        /// Human-readable error message
        message: String,
        /// File path that caused the error
        path: String,
    },

    /// Configuration loading error
    #[error("Failed to load configuration: {message}")]
    LoadError {
        /// Human-readable error message
        message: String,
        /// Source of the configuration (file path, environment, etc.)
        config_source: String,
    },

    /// Configuration saving error
    #[error("Failed to save configuration: {message}")]
    SaveError {
        /// Human-readable error message
        message: String,
        /// Destination path where save was attempted
        path: String,
    },
}

/// Result type for configuration operations
pub type ConfigurationResult<T> = Result<T, ConfigurationError>;

impl ConfigurationError {
    /// Create a TOML parse error with location information
    pub fn toml_parse_error(message: impl Into<String>, line: Option<usize>, column: Option<usize>) -> Self {
        Self::TomlParseError {
            message: message.into(),
            line,
            column,
        }
    }

    /// Create a TOML serialize error
    pub fn toml_serialize_error(message: impl Into<String>) -> Self {
        Self::TomlSerializeError {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>, field: Option<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field,
        }
    }

    /// Create a file error
    pub fn file_error(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::FileError {
            message: message.into(),
            path: path.into(),
        }
    }

    /// Create a load error
    pub fn load_error(message: impl Into<String>, config_source: impl Into<String>) -> Self {
        Self::LoadError {
            message: message.into(),
            config_source: config_source.into(),
        }
    }

    /// Create a save error
    pub fn save_error(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::SaveError {
            message: message.into(),
            path: path.into(),
        }
    }

    /// Get a user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            ConfigurationError::TomlParseError { message, line, column } => {
                let mut msg = format!("Configuration file has invalid TOML syntax: {}", message);
                if let (Some(line), Some(column)) = (line, column) {
                    msg.push_str(&format!(" (at line {}, column {})", line, column));
                }
                msg.push_str("\n\nSuggestion: Check your TOML syntax and ensure all strings are properly quoted.");
                msg
            }
            ConfigurationError::TomlSerializeError { message } => {
                format!("Failed to convert configuration to TOML format: {}\n\nSuggestion: Check that all configuration values are serializable.", message)
            }
            ConfigurationError::ValidationError { message, field } => {
                let mut msg = format!("Configuration validation failed: {}", message);
                if let Some(field) = field {
                    msg.push_str(&format!(" (field: {})", field));
                }
                msg.push_str("\n\nSuggestion: Review the configuration documentation and ensure all values are within valid ranges.");
                msg
            }
            ConfigurationError::FileError { message, path } => {
                format!("File operation failed for '{}': {}\n\nSuggestion: Check file permissions and ensure the path exists.", path, message)
            }
            ConfigurationError::LoadError { message, config_source } => {
                format!("Failed to load configuration from '{}': {}\n\nSuggestion: Verify the configuration source is accessible and properly formatted.", config_source, message)
            }
            ConfigurationError::SaveError { message, path } => {
                format!("Failed to save configuration to '{}': {}\n\nSuggestion: Check write permissions and ensure the directory exists.", path, message)
            }
        }
    }

    /// Get error category for programmatic handling
    pub fn category(&self) -> ErrorCategory {
        match self {
            ConfigurationError::TomlParseError { .. } => ErrorCategory::Parse,
            ConfigurationError::TomlSerializeError { .. } => ErrorCategory::Serialize,
            ConfigurationError::ValidationError { .. } => ErrorCategory::Validation,
            ConfigurationError::FileError { .. } => ErrorCategory::Io,
            ConfigurationError::LoadError { .. } => ErrorCategory::Io,
            ConfigurationError::SaveError { .. } => ErrorCategory::Io,
        }
    }
}

/// Error categories for programmatic error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// TOML parsing errors
    Parse,
    /// TOML serialization errors
    Serialize,
    /// Configuration validation errors
    Validation,
    /// File I/O errors
    Io,
}

/// Error handling utilities
pub mod utils {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Load configuration from a TOML file
    pub fn load_from_file<T>(path: impl AsRef<Path>) -> ConfigurationResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigurationError::file_error(
                format!("Failed to read file: {}", e),
                path.display().to_string()
            ))?;

        toml::from_str(&content)
            .map_err(|e| {
                // Try to extract line and column information from TOML error
                let (line, column) = extract_toml_error_location(&e);
                ConfigurationError::toml_parse_error(
                    e.to_string(),
                    line,
                    column
                )
            })
    }

    /// Save configuration to a TOML file
    pub fn save_to_file<T>(config: &T, path: impl AsRef<Path>) -> ConfigurationResult<()>
    where
        T: serde::Serialize,
    {
        let path = path.as_ref();
        let toml_string = toml::to_string_pretty(config)
            .map_err(|e| ConfigurationError::toml_serialize_error(e.to_string()))?;

        fs::write(path, toml_string)
            .map_err(|e| ConfigurationError::file_error(
                format!("Failed to write file: {}", e),
                path.display().to_string()
            ))?;

        Ok(())
    }

    /// Extract line and column information from TOML error
    fn extract_toml_error_location(error: &toml::de::Error) -> (Option<usize>, Option<usize>) {
        // TOML errors don't always include line/column info, but we can try to extract it
        let error_str = error.to_string();
        
        // Look for patterns like "line 5 column 10"
        if let Some(caps) = regex::Regex::new(r"line (\d+) column (\d+)")
            .unwrap()
            .captures(&error_str)
        {
            if let (Ok(line), Ok(column)) = (caps[1].parse(), caps[2].parse()) {
                return (Some(line), Some(column));
            }
        }
        
        // Look for patterns like "at line 5"
        if let Some(caps) = regex::Regex::new(r"at line (\d+)")
            .unwrap()
            .captures(&error_str)
        {
            if let Ok(line) = caps[1].parse() {
                return (Some(line), None);
            }
        }
        
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = ConfigurationError::toml_parse_error("Invalid syntax", Some(5), Some(10));
        assert!(error.to_string().contains("Invalid syntax"));
        assert_eq!(error.category(), ErrorCategory::Parse);
    }

    #[test]
    fn test_user_message() {
        let error = ConfigurationError::validation_error("Value out of range", Some("port".to_string()));
        let message = error.user_message();
        assert!(message.contains("Value out of range"));
        assert!(message.contains("field: port"));
        assert!(message.contains("Suggestion:"));
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(ConfigurationError::toml_parse_error("test", None, None).category(), ErrorCategory::Parse);
        assert_eq!(ConfigurationError::toml_serialize_error("test").category(), ErrorCategory::Serialize);
        assert_eq!(ConfigurationError::validation_error("test", None).category(), ErrorCategory::Validation);
        assert_eq!(ConfigurationError::file_error("test", "path").category(), ErrorCategory::Io);
    }
}
