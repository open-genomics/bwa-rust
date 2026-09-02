//! CLI 二进制级集成测试：运行真实的 `bwa-rust` 可执行文件，
//! 覆盖 `index` / `align` / `mem` 三条命令路径。

use std::path::PathBuf;
use std::process::Command;

/// 定位 data/ 目录下的 fixture 文件（tests 运行目录为 crate 根）
fn data_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("data")
}

/// 在临时目录中生成唯一的输出路径，测试后清理
fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bwa_rust_cli_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bwa-rust"))
        .args(args)
        .output()
        .expect("failed to run bwa-rust binary")
}

#[test]
fn cli_index_builds_fm_file() {
    let fa = data_dir().join("toy.fa");
    let out = temp_path("index_out");
    let output = run(&["index", fa.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "index failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sequences: 3"), "unexpected stdout: {}", stdout);
    assert!(stdout.contains("total_len: 244"), "unexpected stdout: {}", stdout);

    // 生成的 .fm 文件存在且非空
    let fm_path = format!("{}.fm", out.to_str().unwrap());
    let meta = std::fs::metadata(&fm_path).expect("index file not created");
    assert!(meta.len() > 0);
    std::fs::remove_file(&fm_path).ok();
}

#[test]
fn cli_align_outputs_valid_sam() {
    let fa = data_dir().join("toy.fa");
    let fq = data_dir().join("toy_reads.fq");
    let out = temp_path("align_out");
    let sam_path = format!("{}.sam", out.to_str().unwrap());

    // 两步：先建索引，再比对
    let index = run(&["index", fa.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert!(index.status.success());

    let align = run(&[
        "align",
        "-i",
        &format!("{}.fm", out.to_str().unwrap()),
        fq.to_str().unwrap(),
        "-o",
        &sam_path,
    ]);
    assert!(
        align.status.success(),
        "align failed: {}",
        String::from_utf8_lossy(&align.stderr)
    );

    let sam = std::fs::read_to_string(&sam_path).expect("SAM file not created");
    let lines: Vec<&str> = sam.lines().collect();

    // header 行
    assert!(sam.starts_with("@HD\tVN:1.6\tSO:unsorted"));
    assert!(sam.contains("@SQ\tSN:chr1"));
    assert!(sam.contains("@SQ\tSN:chr2"));
    assert!(sam.contains("@SQ\tSN:chr3"));

    // 8 条 read，可能有多比对（secondary/supplementary）输出多行。
    // read8 为纯 T 无法比对，应恰好输出 1 条 unmapped 记录（FLAG=4）。
    let records: Vec<&str> = lines.iter().filter(|l| !l.starts_with('@')).copied().collect();
    assert!(
        records.len() >= 8,
        "expected at least 8 SAM records, got {}",
        records.len()
    );

    let read8_records: Vec<&str> = records.iter().filter(|r| r.starts_with("read8_unmapped")).copied().collect();
    assert_eq!(read8_records.len(), 1, "read8 should produce exactly one record");
    let read8: Vec<&str> = read8_records[0].split('\t').collect();
    assert_eq!(read8[1], "4", "read8 should be unmapped");
    assert_eq!(read8[2], "*");

    // read1 的主比对应精确到 chr1，CIGAR 为 20M
    let read1_primary: Vec<&str> = records
        .iter()
        .filter(|r| r.starts_with("read1"))
        .map(|r| r.split('\t').collect::<Vec<&str>>())
        .find(|f| f[1] == "0")
        .expect("read1 primary alignment not found");
    assert_eq!(read1_primary[2], "chr1");
    assert_eq!(read1_primary[5], "20M");

    std::fs::remove_file(&sam_path).ok();
    std::fs::remove_file(format!("{}.fm", out.to_str().unwrap())).ok();
}

#[test]
fn cli_mem_one_step_align() {
    let fa = data_dir().join("toy.fa");
    let fq = data_dir().join("toy_reads.fq");
    let out = temp_path("mem_out");
    let sam_path = format!("{}.sam", out.to_str().unwrap());

    let output = run(&[
        "mem",
        fa.to_str().unwrap(),
        fq.to_str().unwrap(),
        "-o",
        &sam_path,
        "-t",
        "2",
    ]);
    assert!(
        output.status.success(),
        "mem failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let sam = std::fs::read_to_string(&sam_path).expect("SAM file not created");
    assert!(sam.starts_with("@HD\tVN:1.6\tSO:unsorted"));
    // read8（纯 T）应有一条 unmapped 记录（id 为 read8_unmapped）
    assert!(
        sam.lines().any(|l| l.starts_with("read8_unmapped\t4\t*")),
        "read8 should be unmapped in mem output"
    );

    std::fs::remove_file(&sam_path).ok();
}
