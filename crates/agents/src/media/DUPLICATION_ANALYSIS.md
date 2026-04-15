# Media 模块重复代码分析报告

## 概述

对 `crates/agents/src/media/` 目录下的7个文件进行了重复代码分析，发现了多处可以合并或提取的功能重复。

**文件总览：**
- `attachment.rs` (1235行) - 附件解析
- `downloader.rs` (1077行) - 媒体下载
- `mime.rs` (937行) - MIME类型检测
- `store.rs` (751行) - 媒体存储
- `understanding.rs` (812行) - 媒体理解
- `formatter.rs` (525行) - 消息格式化
- `multimodal.rs` (619行) - 多模态处理

---

## 1. MIME类型处理重复 (HIGH)

### 问题描述
MIME类型与扩展名之间的转换逻辑在4个文件中重复实现。

### 重复代码位置

#### A. `attachment.rs` 第952-976行
```rust
fn guess_mime_type(extension: &str) -> String {
    let ext = extension.trim_start_matches('.').to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        // ... 更多映射
        _ => "application/octet-stream",
    }
}
```

#### B. `downloader.rs` 第747-765行
```rust
fn mime_to_extension(&self, mime_type: &str) -> String {
    match mime_type {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        // ... 更多映射
        _ => "bin",
    }
}
```

#### C. `store.rs` 第615-646行
```rust
fn mime_guess_from_extension(ext: &str) -> String {
    let ext = ext.trim_start_matches('.').to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        // ... 更多映射
        _ => "application/octet-stream",
    }
}
```

#### D. `mime.rs` (完整文件已存在)
已经有一个完整的 `MimeDetector` 结构体，提供了：
- `from_extension()` - 从扩展名获取MIME类型
- `to_extension()` - 从MIME类型获取扩展名
- `detect()` - 从文件magic bytes检测

### 建议重构
**统一使用 `mime.rs` 中的 `MimeDetector`**

```rust
// 替换 attachment.rs 中的 guess_mime_type
use crate::media::mime::MimeDetector;

// 旧的
let mime_type = Self::guess_mime_type(&extension);

// 新的
let mime_type = MimeDetector::from_extension(&extension)
    .unwrap_or("application/octet-stream");
```

---

## 2. 文件扩展名提取重复 (MEDIUM)

### 问题描述
提取文件扩展名的逻辑几乎完全相同，但存在于多个文件中。

### 重复代码位置

#### A. `attachment.rs` 第928-935行
```rust
fn extract_extension(filename: &str) -> String {
    std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default()
}
```

#### B. `store.rs` 第445-451行
```rust
fn extract_extension(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default()
}
```

### 差异分析
- `attachment.rs` 返回 `"jpg"` (无点号)
- `store.rs` 返回 `".jpg"` (有点号)

### 建议重构
**提取到 `mime.rs` 作为公共工具函数：**

```rust
// mime.rs
pub fn extract_extension(filename: &str, include_dot: bool) -> String {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    if include_dot && !ext.is_empty() {
        format!(".{}", ext)
    } else {
        ext
    }
}
```

---

## 3. 文件保存逻辑重复 (MEDIUM)

### 问题描述
文件保存（目录创建、文件写入）逻辑在 `downloader.rs` 和 `store.rs` 中重复。

### 重复代码位置

#### A. `downloader.rs` 第706-744行
```rust
async fn save_file(&self, bytes: &Bytes, file_name: &str, platform: PlatformType, ...) 
    -> Result<PathBuf> {
    // Build directory path
    let mut dir_path = self.config.download_dir.clone();
    if self.config.organize_by_platform { ... }
    if self.config.organize_by_date { ... }
    
    // Create directory
    fs::create_dir_all(&dir_path).await?;
    
    // Write file
    let mut file = fs::File::create(&file_path).await?;
    file.write_all(bytes).await?;
    
    Ok(file_path)
}
```

#### B. `store.rs` 第123-214行
```rust
pub async fn save(&self, data: &[u8], original_name: impl Into<String>, ...) 
    -> Result<MediaMetadata> {
    // Build directory path
    let dir_path = self.build_directory_path();
    
    // Create directory
    fs::create_dir_all(parent_dir).await?;
    
    // Write file
    let mut file = fs::File::create(&absolute_path).await?;
    file.write_all(data).await?;
    file.flush().await?;
    
    Ok(metadata)
}
```

### 建议重构
**`downloader.rs` 应该使用 `MediaStore` 来保存文件：**

```rust
// 在 MediaDownloader 中使用 MediaStore
pub struct MediaDownloader {
    config: DownloadConfig,
    http_client: reqwest::Client,
    lark_config: Option<LarkMediaConfig>,
    download_semaphore: Arc<Semaphore>,
    cache: Option<Arc<MediaCache>>,
    store: MediaStore,  // 添加 store
}

// 保存时委托给 store
async fn save_file(&self, bytes: &Bytes, file_name: &str, ...) -> Result<PathBuf> {
    let metadata = self.store.save(bytes, file_name, mime_type, None).await?;
    Ok(metadata.absolute_path)
}
```

---

## 4. 图片格式检测重复 (MEDIUM)

### 问题描述
通过 magic bytes 检测图片格式的逻辑重复。

### 重复代码位置

#### A. `downloader.rs` 第876-888行
```rust
fn detect_mime_type(data: &[u8]) -> String {
    match &data[0..4] {
        [0xFF, 0xD8, 0xFF, _] => "image/jpeg".to_string(),
        [0x89, 0x50, 0x4E, 0x47] => "image/png".to_string(),
        [0x47, 0x49, 0x46, 0x38] => "image/gif".to_string(),
        [0x52, 0x49, 0x46, 0x46] => "image/webp".to_string(),
        [0x25, 0x50, 0x44, 0x46] => "application/pdf".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
```

#### B. `multimodal.rs` 第489-517行
```rust
fn detect_image_format(&self, bytes: &[u8]) -> Result<ImageFormat> {
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(ImageFormat::Png);
    }
    // JPEG: FF D8 FF
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(ImageFormat::Jpeg);
    }
    // ... GIF, WebP
}
```

#### C. `mime.rs` 第27-348行
已经存在完整的 `MAGIC_PATTERNS` 数组和 `MimeDetector::detect()` 方法。

### 建议重构
**统一使用 `MimeDetector`：**

```rust
// multimodal.rs
use crate::media::mime::MimeDetector;

fn detect_image_format(&self, bytes: &[u8]) -> Result<ImageFormat> {
    let (mime, _) = MimeDetector::detect(bytes)
        .ok_or_else(|| AgentError::platform("Unknown image format"))?;
    
    match mime {
        "image/jpeg" => Ok(ImageFormat::Jpeg),
        "image/png" => Ok(ImageFormat::Png),
        "image/gif" => Ok(ImageFormat::Gif),
        "image/webp" => Ok(ImageFormat::Webp),
        _ => Err(AgentError::platform("Unsupported image format")),
    }
}
```

---

## 5. Lark Token获取重复 (LOW)

### 问题描述
Lark 访问令牌的获取逻辑在多个地方实现。

### 重复代码位置

#### A. `downloader.rs` 第626-665行 (活跃使用)
```rust
async fn get_lark_token(&self) -> Result<String> { ... }
```

#### B. `multimodal.rs` 第116-144行 (标记为 dead_code)
```rust
#[allow(dead_code)]
async fn get_access_token(&self, app_id: &str, app_secret: &str) -> Result<String> { ... }
```

### 建议重构
- 移除 `multimodal.rs` 中未使用的 `get_access_token` 方法
- 或将其委托给 `MediaDownloader`

---

## 6. 附件类型判断重复 (MEDIUM)

### 问题描述
判断附件是否为图片/文档的逻辑在多个文件中重复。

### 重复代码位置

#### A. `attachment.rs` 第72-131行
```rust
impl AttachmentType {
    pub fn from_mime_type(mime_type: &str) -> Self { ... }
    pub fn from_extension(extension: &str) -> Self { ... }
}
```

#### B. `formatter.rs` 第424-437行
```rust
fn is_image_type(&self, attachment_type: &AttachmentType) -> bool {
    matches!(attachment_type, 
        AttachmentType::Image | AttachmentType::EmbeddedImage | AttachmentType::Sticker)
}

fn is_document_type(&self, attachment_type: &AttachmentType) -> bool {
    matches!(attachment_type, AttachmentType::Document | AttachmentType::File)
}
```

#### C. `understanding.rs` 第459-496行
```rust
fn infer_document_type(&self, attachment: &ParsedAttachment) -> DocumentType {
    let mime = attachment.mime_type.as_deref().unwrap_or("");
    let ext = attachment.extension.as_deref().unwrap_or("");
    // 根据 mime/ext 判断文档类型
}
```

### 建议重构
**在 `AttachmentType` 上添加方法：**

```rust
impl AttachmentType {
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image | Self::EmbeddedImage | Self::Sticker)
    }
    
    pub fn is_document(&self) -> bool {
        matches!(self, Self::Document | Self::File)
    }
    
    pub fn is_audio(&self) -> bool {
        matches!(self, Self::Audio)
    }
    
    pub fn is_video(&self) -> bool {
        matches!(self, Self::Video)
    }
}
```

---

## 7. 配置结构重复 (LOW)

### 问题描述
多个配置结构有相似的字段。

| 字段 | `DownloadConfig` | `MediaStoreConfig` |
|------|------------------|-------------------|
| max_file_size | ✅ | ✅ |
| organize_by_date | ✅ | ✅ |
| base_dir/download_dir | ✅ | ✅ |

### 建议重构
考虑提取公共配置 trait 或结构体：

```rust
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub base_dir: PathBuf,
    pub max_file_size: usize,
    pub organize_by_date: bool,
}

// DownloadConfig 和 MediaStoreConfig 都包含 StorageConfig
```

---

## 8. 文件大小检查重复 (LOW)

### 问题描述
文件大小限制的验证逻辑重复。

### 重复代码位置
- `downloader.rs` 第400-405行, 第424-430行
- `store.rs` 第134-141行
- `multimodal.rs` 第307-313行, 第464-470行

### 建议重构
**提取到公共 trait：**

```rust
pub trait SizeLimited {
    fn max_size(&self) -> usize;
    
    fn check_size(&self, size: usize) -> Result<()> {
        if size > self.max_size() {
            Err(AgentError::platform(format!(
                "File too large: {} bytes (max: {})",
                size, self.max_size()
            )))
        } else {
            Ok(())
        }
    }
}
```

---

## 9. URL文件名提取重复 (LOW)

### 问题描述
从URL提取文件名的逻辑重复。

### 重复代码位置
- `attachment.rs` 第938-949行: `extract_filename_from_url`
- `downloader.rs` 第688-697行: 内联在 `extract_filename` 中

### 建议重构
**提取到公共工具模块：**

```rust
// utils.rs 或 mime.rs
pub fn extract_filename_from_url(url: &str) -> Option<String> {
    url::Url::parse(url).ok()
        .and_then(|u| u.path_segments())
        .and_then(|s| s.last())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}
```

---

## 10. 类型转换重复 (MEDIUM)

### 问题描述
`AttachmentType` 和 `MediaType` 之间的转换逻辑分散。

### 重复代码位置

#### A. `attachment.rs` 第72-85行
```rust
impl AttachmentType {
    pub fn to_media_type(&self) -> MediaType {
        match self {
            AttachmentType::Image | AttachmentType::EmbeddedImage => MediaType::Image,
            AttachmentType::File | AttachmentType::Document | AttachmentType::Archive => MediaType::File,
            AttachmentType::Audio => MediaType::Voice,
            AttachmentType::Video => MediaType::Video,
            AttachmentType::Sticker => MediaType::Sticker,
            AttachmentType::Unknown => MediaType::File,
        }
    }
}
```

#### B. `downloader.rs` 第63-78行
```rust
impl MediaType {
    pub fn from_mime_type(mime_type: &str) -> Self {
        if mime_type.starts_with("image/") { ... }
        else if mime_type.starts_with("audio/") { ... }
        ...
    }
}
```

### 建议重构
- 保留 `AttachmentType::to_media_type()`
- `MediaType::from_mime_type()` 可以委托给 `MimeDetector`

---

## 重构优先级建议

### P1 (高优先级)
1. **MIME类型处理** - 统一使用 `MimeDetector`，消除3处重复实现
2. **文件保存逻辑** - 让 `downloader.rs` 使用 `MediaStore`

### P2 (中优先级)
3. **图片格式检测** - 统一使用 `MimeDetector::detect()`
4. **扩展名提取** - 提取公共工具函数
5. **附件类型判断方法** - 在 `AttachmentType` 上添加 `is_image()`, `is_document()` 等方法

### P3 (低优先级)
6. **Lark Token获取** - 清理未使用的代码
7. **配置结构** - 考虑提取公共配置
8. **文件大小检查** - 提取公共 trait
9. **URL文件名提取** - 提取公共工具函数

---

## 建议的新模块结构

```
media/
├── mod.rs              # 模块入口
├── types.rs            # AttachmentType, MediaType, PlatformSource 等类型定义
├── mime.rs             # MimeDetector (已存在，扩展工具函数)
├── storage.rs          # MediaStore (已存在)
├── download.rs         # MediaDownloader (重构，使用 MediaStore)
├── attachment.rs       # AttachmentParser (简化，使用 MimeDetector)
├── understanding.rs    # 媒体理解 (使用新的类型方法)
├── formatter.rs        # 消息格式化 (使用新的类型方法)
└── multimodal.rs       # 多模态处理 (使用 MimeDetector)
```

---

## 预计代码减少量

| 文件 | 当前行数 | 预计减少 | 减少方式 |
|------|---------|---------|---------|
| `attachment.rs` | 1235 | ~100行 | 移除 guess_mime_type, extract_extension |
| `downloader.rs` | 1077 | ~80行 | 移除 save_file, mime_to_extension, detect_mime_type |
| `store.rs` | 751 | ~30行 | 移除 extract_extension, mime_guess_from_extension |
| `multimodal.rs` | 619 | ~30行 | 移除 detect_image_format, get_access_token |
| `formatter.rs` | 525 | ~20行 | 使用 AttachmentType.is_image() 方法 |
| `understanding.rs` | 812 | ~40行 | 使用 MimeDetector |
| **总计** | **5019** | **~300行** | |

预计可以减少约 **6%** 的代码量，同时提高可维护性。
