// Automatically generated Rust module for 'tutorial.proto' file. Do not modify directly.

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use crate::{helpers::*, types::{protobuf::*, MapKey, MessageBuilderBase, MessageBuilder, WireType}, ScratchBuffer, ScratchWriter, Writer};
use super::*;

pub struct Person;

pub struct PersonBuilder<'w, S: ScratchBuffer> {
    writer: &'w mut ScratchWriter<S>
}

impl<'w, S: ScratchBuffer> PersonBuilder<'w, S> {
    pub fn new(writer: &'w mut ScratchWriter<S>) -> Self {
        Self { writer }
    }

    pub fn name(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(10, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn age(&mut self, value: i32) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(16, |w| w.write_int32(value))?;
        Ok(self)
    }

    pub fn email(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(26, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn finish<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, false)
    }

    pub fn finish_length_delimited<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, true)
    }
}

impl<S: ScratchBuffer> MessageBuilderBase<S> for Person {
    type Builder<'a> = PersonBuilder<'a, S> where S: 'a;
}

impl<S: ScratchBuffer> MessageBuilder<S> for Person {
    fn from_writer<'w>(writer: &'w mut ScratchWriter<S>) -> Self::Builder<'w> {
        PersonBuilder::new(writer)
    }
}

pub struct Greeting;

pub struct GreetingBuilder<'w, S: ScratchBuffer> {
    writer: &'w mut ScratchWriter<S>
}

impl<'w, S: ScratchBuffer> GreetingBuilder<'w, S> {
    pub fn new(writer: &'w mut ScratchWriter<S>) -> Self {
        Self { writer }
    }

    pub fn message(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(10, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn sender<F>(&mut self, f: F) -> std::io::Result<&mut Self>
    where
        F: for<'a> FnOnce(&mut tutorial::PersonBuilder<'a, S>) -> std::io::Result<()>
    {
        {
            self.writer.write_tag(18)?;
            self.writer.track_message(|sw| {
              let mut msg_builder = tutorial::PersonBuilder::new(sw);
              f(&mut msg_builder)
            })?;
        }
        Ok(self)
    }

    pub fn finish<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, false)
    }

    pub fn finish_length_delimited<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, true)
    }
}

impl<S: ScratchBuffer> MessageBuilderBase<S> for Greeting {
    type Builder<'a> = GreetingBuilder<'a, S> where S: 'a;
}

impl<S: ScratchBuffer> MessageBuilder<S> for Greeting {
    fn from_writer<'w>(writer: &'w mut ScratchWriter<S>) -> Self::Builder<'w> {
        GreetingBuilder::new(writer)
    }
}

