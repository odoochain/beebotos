//! LLM 响应处理模块
//!
//! 支持文本、图片等多种类型的响应，便于回传给各个平台

use serde::{Deserialize, Serialize};

/// LLM 响应内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmResponseContent {
    /// 纯文本响应
    Text(String),
    /// 图片响应 (base64 编码)
    Image { 
        /// base64 编码的图片数据
        data: String,
        /// MIME 类型
        mime_type: String,
        /// 图片描述
        alt: Option<String>,
    },
    /// 图文混合响应
    Mixed(Vec<LlmResponsePart>),
}

/// LLM 响应部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmResponsePart {
    /// 文本部分
    Text(String),
    /// 图片部分
    Image {
        /// base64 编码的图片数据
        data: String,
        /// MIME 类型
        mime_type: String,
        /// 图片描述
        alt: Option<String>,
    },
    /// 文件部分
    File {
        /// base64 编码的文件数据
        data: String,
        /// 文件名
        name: String,
        /// MIME 类型
        mime_type: String,
    },
}

/// LLM 响应
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// 响应内容
    pub content: LlmResponseContent,
    /// 使用的模型
    pub model: String,
    /// 响应令牌数
    pub tokens: Option<u32>,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

impl LlmResponse {
    /// 创建文本响应
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: LlmResponseContent::Text(text.into()),
            model: String::new(),
            tokens: None,
            success: true,
            error: None,
        }
    }

    /// 创建图片响应
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            content: LlmResponseContent::Image {
                data: data.into(),
                mime_type: mime_type.into(),
                alt: None,
            },
            model: String::new(),
            tokens: None,
            success: true,
            error: None,
        }
    }

    /// 创建混合响应
    pub fn mixed(parts: Vec<LlmResponsePart>) -> Self {
        Self {
            content: LlmResponseContent::Mixed(parts),
            model: String::new(),
            tokens: None,
            success: true,
            error: None,
        }
    }

    /// 创建错误响应
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            content: LlmResponseContent::Text(String::new()),
            model: String::new(),
            tokens: None,
            success: false,
            error: Some(error.into()),
        }
    }

    /// 获取纯文本内容（用于不支持富文本的平台）
    pub fn to_text(&self) -> String {
        match &self.content {
            LlmResponseContent::Text(text) => text.clone(),
            LlmResponseContent::Image { alt, .. } => {
                alt.clone().unwrap_or_else(|| "[图片]".to_string())
            }
            LlmResponseContent::Mixed(parts) => {
                parts.iter().map(|p| match p {
                    LlmResponsePart::Text(t) => t.clone(),
                    LlmResponsePart::Image { alt, .. } => {
                        alt.clone().unwrap_or_else(|| "[图片]".to_string())
                    }
                    LlmResponsePart::File { name, .. } => {
                        format!("[文件: {}]", name)
                    }
                }).collect::<Vec<_>>().join("\n")
            }
        }
    }

    /// 检查是否包含图片
    pub fn has_image(&self) -> bool {
        match &self.content {
            LlmResponseContent::Image { .. } => true,
            LlmResponseContent::Mixed(parts) => {
                parts.iter().any(|p| matches!(p, LlmResponsePart::Image { .. }))
            }
            _ => false,
        }
    }

    /// 获取所有图片
    pub fn get_images(&self) -> Vec<(&str, &str, Option<&str>)> {
        match &self.content {
            LlmResponseContent::Image { data, mime_type, alt } => {
                vec![(data.as_str(), mime_type.as_str(), alt.as_deref())]
            }
            LlmResponseContent::Mixed(parts) => {
                parts.iter()
                    .filter_map(|p| match p {
                        LlmResponsePart::Image { data, mime_type, alt } => {
                            Some((data.as_str(), mime_type.as_str(), alt.as_deref()))
                        }
                        _ => None,
                    })
                    .collect()
            }
            _ => vec![],
        }
    }
}

/// 从 markdown 文本中提取图片
pub fn extract_images_from_markdown(text: &str) -> Vec<(String, String)> {
    let mut images = Vec::new();
    
    // 匹配 ![alt](url) 格式 - 使用static避免重复编译
    static MARKDOWN_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = MARKDOWN_RE.get_or_init(|| {
        regex::Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").expect("Invalid markdown regex")
    });
    
    for cap in re.captures_iter(text) {
        let alt = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let url = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
        if !url.is_empty() {
            images.push((alt, url));
        }
    }
    
    images
}

/// 检测文本中的图片 URL
pub fn detect_image_url(text: &str) -> Option<&str> {
    // 常见的图片扩展名
    let image_extensions = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp"];
    
    // 查找 URL - 使用static避免重复编译
    static URL_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let url_re = URL_RE.get_or_init(|| {
        regex::Regex::new(r"https?://[^\s]+").expect("Invalid URL regex")
    });
    
    for cap in url_re.captures_iter(text) {
        let url = cap.get(0)?.as_str();
        let lower = url.to_lowercase();
        if image_extensions.iter().any(|ext| lower.ends_with(ext)) {
            return Some(url);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_response() {
        let resp = LlmResponse::text("Hello world");
        assert_eq!(resp.to_text(), "Hello world");
        assert!(!resp.has_image());
    }

    #[test]
    fn test_image_response() {
        let resp = LlmResponse::image("base64data", "image/png")
            .content_with_alt("A cat");
        // Note: This test would need adjustment based on actual implementation
    }

    #[test]
    fn test_mixed_response() {
        let parts = vec![
            LlmResponsePart::Text("Here is an image:".to_string()),
            LlmResponsePart::Image {
                data: "base64".to_string(),
                mime_type: "image/png".to_string(),
                alt: Some("A cat".to_string()),
            },
        ];
        let resp = LlmResponse::mixed(parts);
        assert!(resp.has_image());
        assert_eq!(resp.get_images().len(), 1);
    }
}
