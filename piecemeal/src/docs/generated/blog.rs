// Automatically generated Rust module for 'blog.proto' file. Do not modify directly.

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use crate::GenericMapBuilder;
use crate::RepeatedBuilder;
use crate::{helpers::*, types::{protobuf::*, MapKey, MessageBuilderBase, MessageBuilder, WireType}, ScratchBuffer, ScratchWriter, Writer};
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PostStatus {
    DRAFT = 0,
    PUBLISHED = 1,
    ARCHIVED = 2,
}

impl Default for PostStatus {
    fn default() -> Self {
        PostStatus::DRAFT
    }
}

impl From<i32> for PostStatus {
    fn from(i: i32) -> Self {
        match i {
            0 => PostStatus::DRAFT,
            1 => PostStatus::PUBLISHED,
            2 => PostStatus::ARCHIVED,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PostStatus {
    fn from(s: &'a str) -> Self {
        match s {
            "DRAFT" => PostStatus::DRAFT,
            "PUBLISHED" => PostStatus::PUBLISHED,
            "ARCHIVED" => PostStatus::ARCHIVED,
            _ => Self::default(),
        }
    }
}

pub struct Author;

pub struct AuthorBuilder<'w, S: ScratchBuffer> {
    writer: &'w mut ScratchWriter<S>
}

impl<'w, S: ScratchBuffer> AuthorBuilder<'w, S> {
    pub fn new(writer: &'w mut ScratchWriter<S>) -> Self {
        Self { writer }
    }

    pub fn name(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(10, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn email(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(18, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn finish<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, false)
    }

    pub fn finish_length_delimited<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, true)
    }
}

impl<S: ScratchBuffer> MessageBuilderBase<S> for Author {
    type Builder<'a> = AuthorBuilder<'a, S> where S: 'a;
}

impl<S: ScratchBuffer> MessageBuilder<S> for Author {
    fn from_writer<'w>(writer: &'w mut ScratchWriter<S>) -> Self::Builder<'w> {
        AuthorBuilder::new(writer)
    }
}

pub struct Comment;

pub struct CommentBuilder<'w, S: ScratchBuffer> {
    writer: &'w mut ScratchWriter<S>
}

impl<'w, S: ScratchBuffer> CommentBuilder<'w, S> {
    pub fn new(writer: &'w mut ScratchWriter<S>) -> Self {
        Self { writer }
    }

    pub fn author_name(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(10, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn content(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(18, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn timestamp(&mut self, value: i64) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(24, |w| w.write_int64(value))?;
        Ok(self)
    }

    pub fn finish<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, false)
    }

    pub fn finish_length_delimited<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, true)
    }
}

impl<S: ScratchBuffer> MessageBuilderBase<S> for Comment {
    type Builder<'a> = CommentBuilder<'a, S> where S: 'a;
}

impl<S: ScratchBuffer> MessageBuilder<S> for Comment {
    fn from_writer<'w>(writer: &'w mut ScratchWriter<S>) -> Self::Builder<'w> {
        CommentBuilder::new(writer)
    }
}

pub struct Post;

pub struct PostBuilder<'w, S: ScratchBuffer> {
    writer: &'w mut ScratchWriter<S>
}

impl<'w, S: ScratchBuffer> PostBuilder<'w, S> {
    pub fn new(writer: &'w mut ScratchWriter<S>) -> Self {
        Self { writer }
    }

    pub fn title(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(10, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn content(&mut self, value: &str) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(18, |w| w.write_string(value))?;
        Ok(self)
    }

    pub fn author<F>(&mut self, f: F) -> std::io::Result<&mut Self>
    where
        F: for<'a> FnOnce(&mut blog::AuthorBuilder<'a, S>) -> std::io::Result<()>
    {
        {
            self.writer.write_tag(26)?;
            self.writer.track_message(|sw| {
              let mut msg_builder = blog::AuthorBuilder::new(sw);
              f(&mut msg_builder)
            })?;
        }
        Ok(self)
    }

    pub fn tags<F>(&mut self, f: F) -> std::io::Result<&mut Self>
    where
        F: for<'a> FnOnce(&mut RepeatedBuilder<'a, S, Bytes>) -> std::io::Result<()>,
    {
        let mut repeated_builder = RepeatedBuilder::new(4, false,self.writer);
        f(&mut repeated_builder)?;
        Ok(self)
    }

    pub fn add_comments<F>(&mut self, f: F) -> std::io::Result<&mut Self>
    where
        F: for<'a> FnOnce(&mut blog::CommentBuilder<'a, S>) -> std::io::Result<()>
    {
        {
            self.writer.write_tag(42)?;
            self.writer.track_message(|sw| {
              let mut msg_builder = blog::CommentBuilder::new(sw);
              f(&mut msg_builder)
            })?;
        }
        Ok(self)
    }

    pub fn metadata(&mut self) -> GenericMapBuilder<'_, S, Bytes, Bytes> {
        GenericMapBuilder::new(6, self.writer)
    }

    pub fn status(&mut self, value: blog::PostStatus) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(56, |w| w.write_enum(value as i32))?;
        Ok(self)
    }

    pub fn view_count(&mut self, value: i64) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(80, |w| w.write_int64(value))?;
        Ok(self)
    }

    pub fn rating(&mut self, value: f64) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(89, |w| w.write_double(value))?;
        Ok(self)
    }

    pub fn is_featured(&mut self, value: bool) -> std::io::Result<&mut Self> {
        self.writer.write_with_tag(96, |w| w.write_bool(value))?;
        Ok(self)
    }

    pub fn featured_media<F>(&mut self, f: F) -> std::io::Result<&mut Self>
    where
        F: FnOnce(&mut FeaturedMediaOneOfBuilder<'_, S>) -> std::io::Result<()>,
    {
        let mut oneof_builder = FeaturedMediaOneOfBuilder::new(self.writer);
        f(&mut oneof_builder)?;
        Ok(self)
    }

    pub fn finish<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, false)
    }

    pub fn finish_length_delimited<W: Writer>(self, output: &mut W) -> std::io::Result<()> {
        self.writer.finish(output, true)
    }
}

impl<S: ScratchBuffer> MessageBuilderBase<S> for Post {
    type Builder<'a> = PostBuilder<'a, S> where S: 'a;
}

impl<S: ScratchBuffer> MessageBuilder<S> for Post {
    fn from_writer<'w>(writer: &'w mut ScratchWriter<S>) -> Self::Builder<'w> {
        PostBuilder::new(writer)
    }
}

pub struct FeaturedMediaOneOfBuilder<'w, S: ScratchBuffer> {
    writer: &'w mut ScratchWriter<S>
}

impl<'w, S: ScratchBuffer> FeaturedMediaOneOfBuilder<'w, S> {
    pub fn new(writer: &'w mut ScratchWriter<S>) -> Self {
        Self { writer }
    }

    pub fn image_url(&mut self, value: &str) -> std::io::Result<()> {
        self.writer.write_with_tag(66, |w| w.write_string(value))
    }

    pub fn video_url(&mut self, value: &str) -> std::io::Result<()> {
        self.writer.write_with_tag(74, |w| w.write_string(value))
    }
}

