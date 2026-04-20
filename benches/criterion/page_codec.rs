//! Microbenchmarks for page tuple encode/decode.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ecaz::bench_api::{
    CurrentFormatMetadata, DataPage, ItemPointer, MetadataPage, TqElementTuple, TqNeighborTuple,
};

fn make_element_tuple(code_len: usize) -> TqElementTuple {
    TqElementTuple {
        level: 2,
        deleted: false,
        heaptids: vec![
            ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            ItemPointer {
                block_number: 2,
                offset_number: 3,
            },
        ],
        gamma: 0.42,
        neighbortid: ItemPointer {
            block_number: 5,
            offset_number: 1,
        },
        code: vec![0xAB; code_len],
    }
}

fn make_neighbor_tuple(count: u16) -> TqNeighborTuple {
    TqNeighborTuple {
        count,
        tids: (0..count)
            .map(|i| ItemPointer {
                block_number: i as u32 + 10,
                offset_number: 1,
            })
            .collect(),
    }
}

fn bench_element_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("page/element_encode");
    for &code_len in &[192, 768, 1536] {
        let tuple = make_element_tuple(code_len);
        group.throughput(Throughput::Bytes(
            TqElementTuple::encoded_len(code_len) as u64
        ));
        group.bench_function(BenchmarkId::from_parameter(code_len), |b| {
            b.iter(|| tuple.encode());
        });
    }
    group.finish();
}

fn bench_element_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("page/element_decode");
    for &code_len in &[192, 768, 1536] {
        let tuple = make_element_tuple(code_len);
        let encoded = tuple.encode().unwrap();
        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_function(BenchmarkId::from_parameter(code_len), |b| {
            b.iter(|| TqElementTuple::decode(&encoded, code_len));
        });
    }
    group.finish();
}

fn bench_neighbor_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("page/neighbor_encode");
    for &count in &[8u16, 16, 32, 64] {
        let tuple = make_neighbor_tuple(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::from_parameter(count), |b| {
            b.iter(|| tuple.encode());
        });
    }
    group.finish();
}

fn bench_neighbor_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("page/neighbor_decode");
    for &count in &[8u16, 16, 32, 64] {
        let tuple = make_neighbor_tuple(count);
        let encoded = tuple.encode().unwrap();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::from_parameter(count), |b| {
            b.iter(|| TqNeighborTuple::decode(&encoded));
        });
    }
    group.finish();
}

fn bench_metadata_roundtrip(c: &mut Criterion) {
    let metadata = MetadataPage::current_v1_scalar(CurrentFormatMetadata {
        m: 16,
        ef_construction: 128,
        entry_point: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        dimensions: 1536,
        bits: 4,
        max_level: 5,
        seed: 42,
        inserted_since_rebuild: 0,
        persisted_binary_sidecar: false,
    });
    let encoded = metadata.encode();

    c.bench_function("page/metadata_decode", |b| {
        b.iter(|| MetadataPage::decode(&encoded));
    });
}

fn bench_page_insert_read_element(c: &mut Criterion) {
    let mut group = c.benchmark_group("page/insert_read_element");
    let page_size = 8192;
    for &code_len in &[192, 768] {
        let tuple = make_element_tuple(code_len);

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new("insert", format!("code{code_len}")), |b| {
            b.iter_batched(
                || DataPage::new(1, page_size),
                |mut page| page.insert_element(&tuple),
                criterion::BatchSize::SmallInput,
            );
        });

        // Pre-insert one tuple so we can benchmark reads
        let mut page = DataPage::new(1, page_size);
        let tid = page.insert_element(&tuple).unwrap();
        group.bench_function(BenchmarkId::new("read", format!("code{code_len}")), |b| {
            b.iter(|| page.read_element(tid, code_len));
        });
    }
    group.finish();
}

fn bench_page_insert_read_neighbor(c: &mut Criterion) {
    let mut group = c.benchmark_group("page/insert_read_neighbor");
    let page_size = 8192;
    for &count in &[16u16, 32] {
        let tuple = make_neighbor_tuple(count);

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new("insert", format!("count{count}")), |b| {
            b.iter_batched(
                || DataPage::new(1, page_size),
                |mut page| page.insert_neighbor(&tuple),
                criterion::BatchSize::SmallInput,
            );
        });

        let mut page = DataPage::new(1, page_size);
        let tid = page.insert_neighbor(&tuple).unwrap();
        group.bench_function(BenchmarkId::new("read", format!("count{count}")), |b| {
            b.iter(|| page.read_neighbor(tid));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_element_encode,
    bench_element_decode,
    bench_neighbor_encode,
    bench_neighbor_decode,
    bench_metadata_roundtrip,
    bench_page_insert_read_element,
    bench_page_insert_read_neighbor
);
criterion_main!(benches);
