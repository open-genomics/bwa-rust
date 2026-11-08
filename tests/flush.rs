//! 文件输出路径的 flush 回归测试（Linux）。
//!
//! 输出 writer 在最后一次 flush 失败时（如磁盘满、配额超限）必须向上返回错误，
//! 不能以成功退出并留下截断的 SAM。

#![cfg(target_os = "linux")]

use std::io::Cursor;
use std::sync::Arc;

use bwa_rust::align::{align_fastq_with_fm_opt, AlignOpt};
use bwa_rust::index::builder::build_fm_index;

#[test]
fn file_output_flush_error_is_propagated() {
    let fasta = b">chr1\nACGTACGTACGTACGTACGTACGTACGTACGT\n";
    let fm = Arc::new(build_fm_index(Cursor::new(&fasta[..]), 16).unwrap().fm);

    let dir = std::env::temp_dir().join(format!("bwa_rust_flush_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fq_path = dir.join("reads.fq");
    std::fs::write(&fq_path, b"@read1\nACGTACGTACGTACGTACGT\n+\nIIIIIIIIIIIIIIIIIIII\n").unwrap();

    if !std::path::Path::new("/dev/full").exists() {
        eprintln!("skipping: /dev/full not available");
        std::fs::remove_dir_all(&dir).ok();
        return;
    }

    // 单条 read 的输出远小于 BufWriter 缓冲区；若末尾不显式 flush，
    // ENOSPC 会在 Drop 中被吞掉，函数错误地返回 Ok。
    let result = align_fastq_with_fm_opt(fm, fq_path.to_str().unwrap(), Some("/dev/full"), AlignOpt::default());

    std::fs::remove_dir_all(&dir).ok();
    let err = result.expect_err("write failure on /dev/full must not be swallowed");
    let io_err = err
        .downcast_ref::<std::io::Error>()
        .expect("flush failure should surface as io::Error");
    assert_eq!(io_err.kind(), std::io::ErrorKind::StorageFull);
}
