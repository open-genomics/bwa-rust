use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::align::{self, SwParams};
use bwa_rust::util::dna;

fn make_reference(len: usize) -> Vec<u8> {
    let bases = [b'A', b'C', b'G', b'T'];
    let mut seq = Vec::with_capacity(len);
    let mut x: u32 = 42;
    for _ in 0..len {
        x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        seq.push(bases[(x >> 16) as usize % 4]);
    }
    seq
}

fn build_fm_index(seq: &[u8]) -> fm::FMIndex {
    let norm = dna::normalize_seq(seq);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    let len = text.len() as u32;
    let contigs = vec![fm::Contig {
        name: "bench".to_string(),
        len,
        offset: 0,
    }];
    text.push(0);
    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, 128)
}

fn bench_backward_search(c: &mut Criterion) {
    let reference = make_reference(10_000);
    let fm_idx = build_fm_index(&reference);
    let pattern: Vec<u8> = reference[100..120].iter().map(|&b| dna::to_alphabet(b)).collect();

    c.bench_function("backward_search_20bp", |b| {
        b.iter(|| {
            black_box(fm_idx.backward_search(black_box(&pattern)));
        })
    });
}

fn bench_smem_seeds(c: &mut Criterion) {
    let reference = make_reference(10_000);
    let fm_idx = build_fm_index(&reference);
    let read = &reference[500..600];
    let norm = dna::normalize_seq(read);
    let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();

    c.bench_function("smem_seeds_100bp", |b| {
        b.iter(|| {
            black_box(align::find_smem_seeds(black_box(&fm_idx), black_box(&alpha), 19));
        })
    });
}

fn bench_banded_sw(c: &mut Criterion) {
    let query = make_reference(100);
    let mut ref_seq = query.clone();
    ref_seq[50] = b'N'; // introduce mismatch
    let params = SwParams {
        match_score: 2,
        mismatch_penalty: 1,
        gap_open: 2,
        gap_extend: 1,
        band_width: 16,
    };

    c.bench_function("banded_sw_100bp", |b| {
        b.iter(|| {
            black_box(align::banded_sw(black_box(&query), black_box(&ref_seq), params));
        })
    });
}

fn bench_build_sa(c: &mut Criterion) {
    let reference = make_reference(10_000);
    let text: Vec<u8> = dna::normalize_seq(&reference)
        .iter()
        .map(|&b| dna::to_alphabet(b))
        .chain(std::iter::once(0u8))
        .collect();

    c.bench_function("build_sa_10k", |b| {
        b.iter(|| {
            black_box(sa::build_sa(black_box(&text)));
        })
    });
}

criterion_group!(benches, bench_backward_search, bench_smem_seeds, bench_banded_sw, bench_build_sa);
criterion_main!(benches);
