use std::marker::PhantomData;

use utils::range::{Difference, RangeSet, ToRangeSet, UnionMut};
use bytes::{Bytes, BytesMut};
use crate::{json::JsonValue, Span, Spanned};

/// An HTTP header name.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderName(pub(crate) Span<str>);

impl HeaderName {
    /// Returns the header name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.0.offset(offset);
    }
}

impl Spanned<str> for HeaderName {
    fn span(&self) -> &Span<str> {
        &self.0
    }
}

impl ToRangeSet<usize> for HeaderName {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.0.indices.clone()
    }
}

/// An HTTP header value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderValue(pub(crate) Span);

impl HeaderValue {
    /// Returns the header value as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.0.offset(offset);
    }
}

impl Spanned for HeaderValue {
    fn span(&self) -> &Span {
        &self.0
    }
}

impl ToRangeSet<usize> for HeaderValue {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.0.indices.clone()
    }
}

/// An HTTP header, including optional whitespace and the trailing CRLF.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub(crate) span: Span,
    /// The header name.
    pub name: HeaderName,
    /// The header value.
    pub value: HeaderValue,
}

impl Header {
    /// Returns the indices of the header excluding the value.
    ///
    /// The indices will include any optional whitespace and the CRLF.
    pub fn without_value(&self) -> RangeSet<usize> {
        self.span.indices.difference(&self.value.span().indices)
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.span.offset(offset);
        self.name.offset(offset);
        self.value.offset(offset);
    }
}

impl Spanned for Header {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl ToRangeSet<usize> for Header {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.span.indices.clone()
    }
}

/// An HTTP request method.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Method(pub(crate) Span<str>);

impl Method {
    /// Returns the method as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.0.offset(offset);
    }
}

impl Spanned<str> for Method {
    fn span(&self) -> &Span<str> {
        &self.0
    }
}

impl ToRangeSet<usize> for Method {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.0.indices.clone()
    }
}

/// An HTTP request target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Target(pub(crate) Span<str>);

impl Target {
    /// Returns the target as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.0.offset(offset);
    }
}

impl Spanned<str> for Target {
    fn span(&self) -> &Span<str> {
        &self.0
    }
}

impl ToRangeSet<usize> for Target {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.0.indices.clone()
    }
}

/// An HTTP request line, including the trailing CRLF.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RequestLine {
    pub(crate) span: Span<str>,

    /// The request method.
    pub method: Method,
    /// The request target.
    pub target: Target,
}

impl RequestLine {
    /// Returns the indices of the request line excluding the request target.
    pub fn without_target(&self) -> RangeSet<usize> {
        self.span.indices.difference(&self.target.0.indices)
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.span.offset(offset);
        self.method.offset(offset);
        self.target.offset(offset);
    }
}

impl Spanned<str> for RequestLine {
    fn span(&self) -> &Span<str> {
        &self.span
    }
}

impl ToRangeSet<usize> for RequestLine {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.span.indices.clone()
    }
}

/// An HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Request {
    pub(crate) span: Span,
    /// The request line.
    pub request: RequestLine,
    /// Request headers.
    pub headers: Vec<Header>,
    /// Request body.
    pub body: Option<Body>,
    /// The request total length.
    pub total_len: usize,
}

impl Request {
    /// Returns an iterator of request headers with the given name (case-insensitive).
    ///
    /// This method returns an iterator because it is valid for HTTP records to contain
    /// duplicate header names.
    pub fn headers_with_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Header> {
        self.headers
            .iter()
            .filter(|h| h.name.0.as_str().eq_ignore_ascii_case(name))
    }

    /// Returns the indices of the request excluding the target, headers and body.
    pub fn without_data(&self) -> RangeSet<usize> {
        let mut indices = self.span.indices.difference(&self.request.target.0.indices);
        for header in &self.headers {
            indices = indices.difference(header.span.indices());
        }
        if let Some(body) = &self.body {
            indices = indices.difference(body.span.indices());
        }
        indices
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.span.offset(offset);
        self.request.offset(offset);
        for header in &mut self.headers {
            header.offset(offset);
        }
        if let Some(body) = &mut self.body {
            body.offset(offset);
        }
    }
}

impl Spanned for Request {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl ToRangeSet<usize> for Request {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.span.indices.clone()
    }
}

/// An HTTP response code.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Code(pub(crate) Span<str>);

impl Code {
    /// Returns the response code as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.0.offset(offset);
    }
}

impl Spanned<str> for Code {
    fn span(&self) -> &Span<str> {
        &self.0
    }
}

impl ToRangeSet<usize> for Code {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.0.indices.clone()
    }
}

/// An HTTP response reason phrase.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reason(pub(crate) Span<str>);

impl Reason {
    /// Returns the response reason phrase as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.0.offset(offset);
    }
}

impl Spanned<str> for Reason {
    fn span(&self) -> &Span<str> {
        &self.0
    }
}

impl ToRangeSet<usize> for Reason {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.0.indices.clone()
    }
}

/// An HTTP response status.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Status {
    pub(crate) span: Span<str>,

    /// The response code.
    pub code: Code,
    /// The reason phrase.
    pub reason: Reason,
}

impl Status {
    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.span.offset(offset);
        self.code.offset(offset);
        self.reason.offset(offset);
    }
}

impl Spanned<str> for Status {
    fn span(&self) -> &Span<str> {
        &self.span
    }
}

impl ToRangeSet<usize> for Status {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.span.indices.clone()
    }
}

/// An HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Response {
    pub(crate) span: Span,
    /// The response status.
    pub status: Status,
    /// Response headers.
    pub headers: Vec<Header>,
    /// Response body.
    pub body: Option<Body>,
    /// The response total length.
    pub total_len: usize,
}

impl Response {
    /// Returns an iterator of response headers with the given name (case-insensitive).
    ///
    /// This method returns an iterator because it is valid for HTTP records to contain
    /// duplicate header names.
    pub fn headers_with_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Header> {
        self.headers
            .iter()
            .filter(|h| h.name.0.as_str().eq_ignore_ascii_case(name))
    }

    /// Returns the indices of the response excluding the headers and body.
    pub fn without_data(&self) -> RangeSet<usize> {
        let mut indices = self.span.indices.clone();
        for header in &self.headers {
            indices = indices.difference(header.span.indices());
        }
        if let Some(body) = &self.body {
            indices = indices.difference(body.span.indices());
        }
        indices
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.span.offset(offset);
        self.status.offset(offset);
        for header in &mut self.headers {
            header.offset(offset);
        }
        if let Some(body) = &mut self.body {
            body.offset(offset);
        }
    }
}

impl Spanned for Response {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl ToRangeSet<usize> for Response {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.span.indices.clone()
    }
}

/// An HTTP request or response payload body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Body {
    pub(crate) span: Span,

    /// The body content.
    pub content: BodyContent,
}

impl Body {
    /// Returns the body as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.span.as_bytes()
    }

    /// Shifts the span range by the given offset.
    pub fn offset(&mut self, offset: usize) {
        self.span.offset(offset);
    }
}

impl Spanned for Body {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl ToRangeSet<usize> for Body {
    fn to_range_set(&self) -> RangeSet<usize> {
        self.span.indices.clone()
    }
}

/// An HTTP request or response payload body content.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum BodyContent {
    /// Body with an `application/json` content type.
    Json(JsonValue),
    /// Body with an unknown content type.
    Unknown(Span),
    /// Body with a `Transfer-Encoding: chunked` header.
    Chunked(ChunkedBody),
}
        

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkedBody {
    pub chunks: Vec<Chunk>,
    pub span: Span,
}

/// A chunk of a chunked body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// The chunk span.
    pub span: Span,
    /// The chunk data.
    pub data: Bytes,
    /// The chunk extension, if any.
    pub extension: Option<Span>,
}

impl Spanned for BodyContent {
    fn span(&self) -> &Span {
        match self {
            BodyContent::Json(json) => json.span().as_ref(),
            BodyContent::Unknown(span) => span,
            BodyContent::Chunked(chunked_body) => &chunked_body.span,
        }
    }
}

impl ToRangeSet<usize> for BodyContent {
    fn to_range_set(&self) -> RangeSet<usize> {
        match self {
            BodyContent::Json(json) => json.span().indices.clone(),
            BodyContent::Unknown(span) => span.indices.clone(),
            BodyContent::Chunked(chunked_body) => chunked_body.span.indices.clone(),
        }
    }
}

fn chunked_body_range_set(chunks: &[Chunk]) -> RangeSet<usize> {
    let mut range_set: RangeSet<usize> = RangeSet::new(&[]);
    for chunk in chunks {
        range_set.union_mut(&chunk.span.indices);
    }
    range_set
}

fn chunked_body_span(chunks: &[Chunk]) -> Span {
    let combined_range_set = chunked_body_range_set(chunks);
    
    let mut combined_data = BytesMut::new();
    for chunk in chunks {
        combined_data.extend_from_slice(&chunk.span.data);
    }
    let combined_data = combined_data.freeze();

    Span {
        data: combined_data,
        indices: combined_range_set,
        _pd: PhantomData,
    }
}

pub(crate) fn build_chunked_body(chunks: &[Chunk]) -> ChunkedBody {
    ChunkedBody {
        chunks: chunks.to_vec(),
        span: chunked_body_span(chunks),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_build_chunked_body() {

        let chunk_data1 = Bytes::from("aabbccc");
        let chunk_data2 = Bytes::from("deeeeffg");
        let src = Bytes::from("aabbcccdeeeeffg");
        let offset1 = chunk_data1.len();

        println!("chunk_data1: {:?}", chunk_data1.as_ref());
        println!("chunk_data2: {:?}", chunk_data2.as_ref());

        // Create some sample chunks
        let chunk1 = Chunk {
            span: Span::new_bytes(src.clone(), 2..4),
            data: chunk_data1,
            extension: None,
        };

        println!("chunk1: {:?}", chunk1);

        let chunk2 = Chunk {
            span: Span::new_bytes(src.clone(), offset1+1..offset1 + 5),
            data: chunk_data2,
            extension: None,
        };


        println!("chunk2: {:?}", chunk2);
        // Create a ChunkedBody with these chunks
        let chunked_body = build_chunked_body(&[chunk1, chunk2]);
        let body_content = BodyContent::Chunked(chunked_body);

        // Get the range set from the ChunkedBody
        let range_set = body_content.span().indices.clone();

        println!("body_content: {:?}", body_content);
        println!("range_set: {:?}", range_set);


        assert_eq!(body_content.span(), b"bbeeee".as_slice());
    }
}
