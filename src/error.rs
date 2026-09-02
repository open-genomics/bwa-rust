use std::fmt;
use std::io;

/// bwa-rust 库级错误类型
///
/// 为库模式调用提供结构化错误信息，便于外部消费者处理特定错误场景。
/// CLI 入口仍可通过 `anyhow` 统一上报。
#[derive(Debug)]
pub enum BwaError {
    /// IO 错误（文件读写、路径不存在等）
    Io(io::Error),
    /// 索引格式错误（magic 不匹配、版本不兼容、校验和不符等）
    IndexFormat(String),
    /// 索引构建错误（空输入、无效序列等）
    IndexBuild(String),
    /// 对齐错误（参数无效、内部逻辑异常等）
    Align(String),
    /// 文件解析错误（FASTA/FASTQ 格式不正确）
    Parse(String),
    /// 解析错误，附带记录位置（record 序号），用于定位损坏的大文件
    ParseAt { record: u64, message: String },
}

impl fmt::Display for BwaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BwaError::Io(e) => write!(f, "IO error: {}", e),
            BwaError::IndexFormat(msg) => write!(f, "Index format error: {}", msg),
            BwaError::IndexBuild(msg) => write!(f, "Index build error: {}", msg),
            BwaError::Align(msg) => write!(f, "Alignment error: {}", msg),
            BwaError::Parse(msg) => write!(f, "Parse error: {}", msg),
            BwaError::ParseAt { record, message } => write!(f, "Parse error at record {}: {}", record, message),
        }
    }
}

impl std::error::Error for BwaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BwaError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BwaError {
    fn from(e: io::Error) -> Self {
        BwaError::Io(e)
    }
}

impl From<anyhow::Error> for BwaError {
    fn from(e: anyhow::Error) -> Self {
        BwaError::Align(e.to_string())
    }
}

/// 库级 Result 类型别名
pub type BwaResult<T> = Result<T, BwaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let e = BwaError::IndexFormat("bad magic".to_string());
        assert!(e.to_string().contains("Index format error"));
        assert!(e.to_string().contains("bad magic"));
    }

    #[test]
    fn error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let e: BwaError = io_err.into();
        assert!(matches!(e, BwaError::Io(_)));
        assert!(e.to_string().contains("file not found"));
    }

    #[test]
    fn error_source() {
        let io_err = io::Error::other("test");
        let e = BwaError::Io(io_err);
        assert!(std::error::Error::source(&e).is_some());

        let e2 = BwaError::Align("test".into());
        assert!(std::error::Error::source(&e2).is_none());
    }

    #[test]
    fn error_parse_at_display() {
        let e = BwaError::ParseAt {
            record: 42,
            message: "bad header".to_string(),
        };
        assert!(e.to_string().contains("record 42"));
        assert!(e.to_string().contains("bad header"));
    }

    #[test]
    fn error_from_anyhow() {
        let e: BwaError = anyhow::anyhow!("inner").into();
        assert!(e.to_string().contains("inner"));
    }
}
