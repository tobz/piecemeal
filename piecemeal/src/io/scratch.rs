//! Scratch buffers and writer.

use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{helpers::sizeof_varint, ProtoResult};

use super::writer::Writer;

/// A scratch buffer.
///
/// Scratch buffers are used to temporarily write message fields as a message is being built, so that it can be written
/// in its entirety to an output stream later.
///
/// `ScratchBuffer` is mostly a superset of `Writer`, but it also provides methods to clear the buffer and read the
/// written bytes to it.
pub trait ScratchBuffer: Writer {
    /// Clears the buffer, removing all values.
    ///
    /// This method has no effect on the allocated capacity of the buffer.
    fn clear(&mut self);

    /// Extracts a slice containing the entire buffer.
    fn as_slice(&self) -> &[u8];
}

impl ScratchBuffer for Vec<u8> {
    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn as_slice(&self) -> &[u8] {
        &self[..]
    }
}

impl<'a> ScratchBuffer for &'a mut Vec<u8> {
    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn as_slice(&self) -> &[u8] {
        &self[..]
    }
}

struct LengthMarker {
    offset: usize,
    len: u64,
}

impl Eq for LengthMarker {}

impl PartialEq for LengthMarker {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
    }
}

impl PartialOrd for LengthMarker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LengthMarker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.offset.cmp(&other.offset)
    }
}

/// A scratch writer.
///
/// When encoding messages, each message needs to be able to specify its own length -- the number of overall bytes in
/// the message -- before any of the message content itself. This is hard to do when building messages one field at a
/// time, since not all fields (and their individual sizes) can be known beforehand.
///
/// We compensate for this by writing into a scratch buffer, a temporary buffer where we can write messages, and their
/// individual fields, without needing to worry about their length. However, this alone is not sufficient for properly
/// writing out the final encoded message, and `ScratchWriter<B>` provides the logic to calculate these lengths and then
/// insert them at proper locations in the final encoded message as it is written to an output stream.
///
/// `ScratchWriter<B>` allows callers to specify when a message is about to be written, so that a marker can be
/// generated, which allows the length of that message to be calculated and stored for final assembly afterwards. When
/// `finalize` is called, the scratch buffer is written to the output stream, while the length markers are used to know
/// when to write the length of each message in between subslices of the scratch buffer.
pub struct ScratchWriter<B> {
    buffer: B,
    total_len_bytes: usize,

    // We use `Reverse` so that when we get the "max" item in the heap, it's the lowest offset, ensuring we pop length
    // markers from lowest offset to highest (i.e. in order).
    len_markers: BinaryHeap<Reverse<LengthMarker>>,
}

impl<B: ScratchBuffer> ScratchWriter<B> {
    /// Create a new `ScratchWriter<B>` with the given buffer.
    pub fn new(buffer: B) -> Self {
        Self {
            buffer,
            total_len_bytes: 0,
            len_markers: BinaryHeap::new(),
        }
    }

    /// Returns `true` if the scratch buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.as_slice().is_empty()
    }

    /// Returns the length of the scratch buffer, including yet-to-be-written length markers.
    ///
    /// This does _not_ include the length of the buffer if a varint-encoded length delimiter was added.
    pub fn len(&self) -> usize {
        self.buffer.as_slice().len() + self.total_len_bytes
    }

    /// Tracks the given operation.
    pub fn track_message<F>(&mut self, f: F) -> ProtoResult<()>
    where
        F: FnOnce(&mut Self) -> ProtoResult<()>,
    {
        // Track the before/after size of the message in the scratch buffer, including how many length bytes were
        // written for any submessages.
        let start_len = self.buffer.as_slice().len();
        let start_total_len_bytes = self.total_len_bytes;
        f(self)?;
        let end_len = self.buffer.as_slice().len();
        let end_total_len_bytes = self.total_len_bytes;

        // Calculate the number of bytes written after calling `f`, along with how many length bytes will need to be
        // written for any submessages. We use that to work backwards to determine how many additional bytes (for the
        // yet-be-written length markers) we need to add to our _own_ length marker.
        let total_len_bytes_delta = end_total_len_bytes - start_total_len_bytes;
        let len_delta = (end_len - start_len) + total_len_bytes_delta;
        self.total_len_bytes += sizeof_varint(len_delta as u64);

        // Insert a length marker for this message.
        self.len_markers.push(Reverse(LengthMarker {
            offset: start_len,
            len: len_delta as u64,
        }));
        Ok(())
    }

    /// Finalizes the scratch buffer, writing it to the given writer.
    ///
    /// This will write out the scratch buffer to the given writer, inserting the necessary length markers for embedded
    /// messages as needed. If `write_length_delimiter` is `true`, the data in the scratch buffer will be prefixed with
    /// the total length of that data as a varint.
    ///
    /// # Errors
    ///
    /// If there is an error writing the scratch buffer into the given writer, an error will be returned.
    pub fn finish<W>(&mut self, writer: &mut W, write_length_delimiter: bool) -> ProtoResult<()>
    where
        W: Writer,
    {
        let mut start = 0;
        let buf = self.buffer.as_slice();

        // If requested, write the overall length for the scratch buffer, before we iterate any length markers.
        if write_length_delimiter {
            let total_len = self.total_len_bytes + buf.len();
            writer.write_varint(total_len as u64)?;
        }

        // Process all of the collected length markers.
        //
        // We write all data from the scratch buffer that comes between the last marker (or 0) and the current marker,
        // then the length value itself, and then update our start offset to process the next length marker in the same
        // way.
        while let Some(Reverse(LengthMarker { offset, len })) = self.len_markers.pop() {
            let sub_buf = &buf[start..offset];

            writer.pb_write_all(sub_buf)?;
            writer.write_varint(len)?;
            start += sub_buf.len();
        }

        // Write whatever of the scratch buffer remains after the last length marker.
        let final_sub_buf = &buf[start..];
        if !final_sub_buf.is_empty() {
            writer.pb_write_all(final_sub_buf)?;
        }

        // Clear the buffer, and internal state, to prepare the writer for subsequent use.
        self.buffer.clear();
        self.total_len_bytes = 0;

        Ok(())
    }
}

impl<B: ScratchBuffer> Writer for ScratchWriter<B> {
    fn pb_write_u8(&mut self, x: u8) -> ProtoResult<()> {
        self.buffer.pb_write_u8(x)
    }

    fn pb_write_u32(&mut self, x: u32) -> ProtoResult<()> {
        self.buffer.pb_write_u32(x)
    }

    fn pb_write_i32(&mut self, x: i32) -> ProtoResult<()> {
        self.buffer.pb_write_i32(x)
    }

    fn pb_write_f32(&mut self, x: f32) -> ProtoResult<()> {
        self.buffer.pb_write_f32(x)
    }

    fn pb_write_u64(&mut self, x: u64) -> ProtoResult<()> {
        self.buffer.pb_write_u64(x)
    }

    fn pb_write_i64(&mut self, x: i64) -> ProtoResult<()> {
        self.buffer.pb_write_i64(x)
    }

    fn pb_write_f64(&mut self, x: f64) -> ProtoResult<()> {
        self.buffer.pb_write_f64(x)
    }

    fn pb_write_all(&mut self, buf: &[u8]) -> ProtoResult<()> {
        self.buffer.pb_write_all(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{helpers::tag, types::WireType, Writer};

    fn varint_field(field_number: u32) -> u32 {
        tag(field_number, WireType::Varint)
    }

    fn msg_field(field_number: u32) -> u32 {
        tag(field_number, WireType::LengthDelimited)
    }

    fn get_length_delimited_buf(buf: &[u8]) -> Vec<u8> {
        let len = buf.len();
        let mut len_buf = Vec::new();
        len_buf.write_varint(len as u64).unwrap();
        len_buf.pb_write_all(buf).unwrap();
        len_buf
    }

    #[test]
    fn sequential() {
        // Our imaginary message struct looks like this highly-compressed pseudo-definition:
        //
        // message A { B b, C c } message B { int64 a } message C { int64 a }
        //
        // We approximate writing out message A by writing out the two embedded submessages in order.
        //
        // Our goal is to end up with an equivalent output as if we used the generated message structs directly and
        // serialized message A, both with and without the varint length delimiter.

        // Create a reusable writer closure so we can generate the same message structure with and without the length
        // delimiter:
        let msg_writer = |w: &mut ScratchWriter<Vec<u8>>| {
            // write: total len bytes = 0
            // finish: total len bytes = 2

            // Start writing A->b:
            w.write_tag(msg_field(1)).unwrap();
            w.track_message(|w| {
                // Write B->a:
                w.write_with_tag(varint_field(1), |w| w.write_uint64(666))
            })
            .unwrap();

            // write: total len bytes = 1
            // finish: total len bytes = 1

            // Start writing A->c:
            w.write_tag(msg_field(2)).unwrap();
            w.track_message(|w| {
                // Write C->a:
                w.write_with_tag(varint_field(1), |w| w.write_uint64(999))
            })
            .unwrap();

            // write: total len bytes = 2
            // finish: total len bytes = 0
        };

        let mut writer = ScratchWriter::new(Vec::new());

        // Write the message without the length delimiter and make sure it's right:
        msg_writer(&mut writer);
        let mut actual_no_delimiter = Vec::new();
        writer.finish(&mut actual_no_delimiter, false).unwrap();

        let expected_no_delimiter = &[0x0a, 0x03, 0x08, 0x9a, 0x05, 0x12, 0x03, 0x08, 0xe7, 0x07];
        assert_eq!(&expected_no_delimiter[..], &actual_no_delimiter[..]);

        // Write the message with the length delimiter and make sure it's right:
        msg_writer(&mut writer);
        let mut actual_delimiter = Vec::new();
        writer.finish(&mut actual_delimiter, true).unwrap();

        let expected_delimiter = get_length_delimited_buf(expected_no_delimiter);
        assert_eq!(&expected_delimiter[..], &actual_delimiter[..]);
    }

    #[test]
    fn nested() {
        // Our imaginary message struct looks like this highly-compressed pseudo-definition:
        //
        // message A { int64 a, int64 b, B c } message B { int64 a, int64 b, C c } message C { int64 a, int64 b }
        //
        // We approximate writing out message A, and all of its subfields, which then includes message B and all of its
        // subfields, and so on.
        //
        // Our goal is to end up with an equivalent output as if we used the generated message structs directly and
        // serialized message A, both with and without the varint length delimiter.

        // Create a reusable writer closure so we can generate the same message structure with and without the length
        // delimiter:
        let msg_writer = |w: &mut ScratchWriter<Vec<u8>>| {
            // Write A->a and A->b and then start writing A->c:
            w.write_with_tag(varint_field(1), |w| w.write_uint64(42))
                .unwrap();
            w.write_with_tag(varint_field(2), |w| w.write_uint64(369))
                .unwrap();
            w.write_tag(msg_field(3)).unwrap();
            w.track_message(|w| {
                // Write B->a and B->b and then start writing B->c:
                w.write_with_tag(varint_field(1), |w| w.write_uint64(27))
                    .unwrap();
                w.write_with_tag(varint_field(2), |w| w.write_uint64(309))
                    .unwrap();
                w.write_tag(msg_field(3)).unwrap();
                w.track_message(|w| {
                    // Write C->a and C->b:
                    w.write_with_tag(varint_field(1), |w| w.write_uint64(88))
                        .unwrap();
                    w.write_with_tag(varint_field(2), |w| w.write_uint64(365))
                })
            })
            .unwrap();
        };

        let mut writer = ScratchWriter::new(Vec::new());

        // Write the message without the length delimiter and make sure it's right:
        msg_writer(&mut writer);
        let mut actual_no_delimiter = Vec::new();
        writer.finish(&mut actual_no_delimiter, false).unwrap();

        let expected_no_delimiter = &[
            0x08, 0x2a, 0x10, 0xf1, 0x02, 0x1a, 0x0c, 0x08, 0x1b, 0x10, 0xb5, 0x02, 0x1a, 0x05,
            0x08, 0x58, 0x10, 0xed, 0x02,
        ];
        assert_eq!(&expected_no_delimiter[..], &actual_no_delimiter[..]);

        // Write the message with the length delimiter and make sure it's right:
        msg_writer(&mut writer);
        let mut actual_delimiter = Vec::new();
        writer.finish(&mut actual_delimiter, true).unwrap();

        let expected_delimiter = get_length_delimited_buf(expected_no_delimiter);
        assert_eq!(&expected_delimiter[..], &actual_delimiter[..]);
    }
}
