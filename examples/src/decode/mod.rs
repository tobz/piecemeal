use piecemeal::ScratchWriter;

mod protos {
    include!(concat!(env!("OUT_DIR"), "/protos/piecemeal/mod.rs"));
}
use self::protos::metrics_example::{
    MetricPayloadBuilder, MetricPayloadDecoded, metric_payload::AggregationType,
};

fn main() {
    // --- Encode a MetricPayload using the piecemeal builder ---

    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut payload_builder = MetricPayloadBuilder::new(&mut scratch_writer);
    payload_builder
        .add_series(|series| {
            series
                .name("cpu.usage")?
                .tags(|tb| tb.add_many(["host:web-1", "env:prod"]))?
                .add_points(|pt| {
                    pt.unix_timestamp(1700000000)?.value(72.5)?;
                    Ok(())
                })?
                .add_points(|pt| {
                    pt.unix_timestamp(1700000060)?.value(68.3)?;
                    Ok(())
                })?
                .aggregation_type(AggregationType::CUMULATIVE)?
                .metadata(|md| {
                    md.origin_product_id(1)?.origin_service_id(42)?;
                    Ok(())
                })?;

            let mut labels = series.labels();
            labels.write_entry("region", "us-east-1")?;
            labels.write_entry("team", "platform")?;

            Ok(())
        })
        .unwrap()
        .add_series(|series| {
            series
                .name("mem.used")?
                .tags(|tb| tb.add_many(["host:web-1", "env:prod"]))?
                .add_points(|pt| {
                    pt.unix_timestamp(1700000000)?.value(4096.0)?;
                    Ok(())
                })?
                .aggregation_type(AggregationType::DELTA)?;
            Ok(())
        })
        .unwrap();

    let mut encoded = Vec::new();
    payload_builder.finish(&mut encoded).unwrap();

    println!("Encoded {} bytes", encoded.len());
    println!();

    // --- Decode using the generated Decoded struct ---

    let payload = MetricPayloadDecoded::decode(&encoded).unwrap();

    // Repeated message field: iterate over series lazily (re-scans the buffer).
    for (i, series) in payload.series().enumerate() {
        let series = series.unwrap();

        // Singular scalar field: getter method, zero-copy &str.
        println!("Series {}: name={}", i, series.name());

        // Enum field: getter method.
        println!("  aggregation_type={:?}", series.aggregation_type());

        // Repeated string field: lazy iterator.
        let tags: Vec<&str> = series.tags().collect::<Result<_, _>>().unwrap();
        println!("  tags={:?}", tags);

        // Repeated message field: lazy iterator over nested messages.
        for point in series.points() {
            let point = point.unwrap();
            // Nested message scalar fields: getter methods.
            println!(
                "  point: ts={} value={}",
                point.unix_timestamp(),
                point.value()
            );
        }

        // Singular message field: decoded lazily via accessor.
        // Returns defaults if the field was never set.
        let metadata = series.metadata().unwrap();
        if metadata.origin_product_id() != 0 || metadata.origin_service_id() != 0 {
            println!(
                "  metadata: product_id={} service_id={}",
                metadata.origin_product_id(),
                metadata.origin_service_id()
            );
        }

        // Map field: lazy iterator yielding (key, value) pairs.
        let labels: Vec<(&str, &str)> = series.labels().collect::<Result<_, _>>().unwrap();
        if !labels.is_empty() {
            println!("  labels:");
            for (k, v) in &labels {
                println!("    {}={}", k, v);
            }
        }

        println!();
    }
}
