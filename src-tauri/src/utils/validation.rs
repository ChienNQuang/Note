use crate::errors::AppError;

/// Validate that a string is not empty or whitespace-only
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::MissingRequiredField(field_name.to_string()));
    }
    Ok(())
}

/// Validate that a string doesn't exceed maximum length
pub fn validate_max_length(value: &str, max_len: usize, field_name: &str) -> Result<(), AppError> {
    if value.len() > max_len {
        return Err(AppError::InvalidBlockData(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_len
        )));
    }
    Ok(())
}

/// Validate page title
pub fn validate_page_title(title: &str) -> Result<(), AppError> {
    validate_not_empty(title, "page title")?;
    validate_max_length(title, 200, "page title")?;
    Ok(())
}

/// Validate block text content  
pub fn validate_block_text(text: &str) -> Result<(), AppError> {
    // Allow empty blocks (user might be typing)
    validate_max_length(text, 10000, "block text")?;
    Ok(())
}

/// Validate UUID format
pub fn validate_uuid_format(id: &str, field_name: &str) -> Result<(), AppError> {
    if !super::is_valid_uuid(id) {
        return Err(AppError::InvalidBlockData(format!(
            "Invalid UUID format for {}",
            field_name
        )));
    }
    Ok(())
}

/// Sanitize text input to prevent issues
pub fn sanitize_text(text: &str) -> String {
    // Remove null bytes and control characters (except newlines and tabs)
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello", "test").is_ok());
        assert!(validate_not_empty("", "test").is_err());
        assert!(validate_not_empty("   ", "test").is_err());
    }

    #[test]
    fn test_validate_max_length() {
        assert!(validate_max_length("hello", 10, "test").is_ok());
        assert!(validate_max_length("hello world!", 5, "test").is_err());
    }

    #[test]
    fn test_validate_page_title() {
        assert!(validate_page_title("My Page").is_ok());
        assert!(validate_page_title("").is_err());
        assert!(validate_page_title(&"x".repeat(201)).is_err());
    }

    #[test]
    fn test_sanitize_text() {
        let input = "Hello\x00World\nNew Line\tTab";
        let output = sanitize_text(input);
        assert_eq!(output, "HelloWorld\nNew Line\tTab");
    }
} 