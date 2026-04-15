//! Security utilities for XSS protection and input sanitization

/// Escape HTML special characters to prevent XSS attacks
pub fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            '/' => output.push_str("&#x2F;"),
            '`' => output.push_str("&#x60;"),
            '=' => output.push_str("&#x3D;"),
            _ => output.push(ch),
        }
    }
    output
}

/// Escape HTML attribute values
pub fn escape_html_attribute(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            '`' => output.push_str("&#x60;"),
            '=' => output.push_str("&#x3D;"),
            '\n' => output.push_str("&#xA;"),
            '\r' => output.push_str("&#xD;"),
            '\t' => output.push_str("&#x9;"),
            _ => output.push(ch),
        }
    }
    output
}

/// Sanitize URL to prevent javascript: protocol injection
pub fn sanitize_url(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();

    // Block dangerous protocols
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("file:")
    {
        return None;
    }

    // Allow safe protocols or relative URLs
    let is_safe = lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("/")
        || lower.starts_with("#")
        || !lower.contains(":");

    if is_safe {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Check if input contains potentially dangerous HTML
pub fn contains_dangerous_html(input: &str) -> bool {
    let lower = input.to_lowercase();
    let patterns = [
        "<script",
        "javascript:",
        "onerror=",
        "onload=",
        "onclick=",
        "eval(",
        "<iframe",
        "<object",
    ];
    patterns.iter().any(|p| lower.contains(p))
}

/// Sanitize filename to prevent directory traversal
pub fn sanitize_filename(input: &str) -> Option<String> {
    let sanitized: String = input
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | '\0'))
        .collect();

    if sanitized.contains("..") || sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}
