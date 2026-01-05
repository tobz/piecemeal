use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use piecemeal::ScratchWriter;

mod protos {
    include!(concat!(env!("OUT_DIR"), "/protos/piecemeal/mod.rs"));
}
use self::protos::metrics_example::{MetricPayloadBuilder, metric_payload::AggregationType};

fn main() {
    let scratch_buf = Vec::with_capacity(1024);
    let mut scratch_writer = ScratchWriter::new(scratch_buf);

    let mut payload_builder = MetricPayloadBuilder::new(&mut scratch_writer);
    payload_builder
        .add_series(|series_builder| {
            series_builder
                .name("metric.name")?
                .tags(|tb| tb.add_many(["tag1", "tag2"]))?
                .add_points(|pb| {
                    pb.unix_timestamp(1234567890)?.value(42.0)?;
                    Ok(())
                })?
                .aggregation_type(AggregationType::CUMULATIVE)?;

            let mut labels = series_builder.labels();
            labels.write_entry("label1", "value1")?;

            Ok(())
        })
        .expect("should not fail to build series");

    let mut output_buf = Vec::new();
    payload_builder
        .finish(&mut output_buf)
        .expect("should not fail to finish");

    println!("encoded: {}", Base64Display::new(&output_buf, &STANDARD));
}
