use crate::http::{types::{Chunk, ChunkedBody, Header}, Body, BodyContent, ParseError};
use bytes::Bytes;
use std::ops::Range;
use crate::http::types::build_chunked_body;

use crate::{json, Span};

// Parse length of the body from the Content-Length header.
pub(crate) fn parse_content_length(h: &Header) -> Result<usize, ParseError> {
    std::str::from_utf8(h.value.0.as_bytes())?
        .parse::<usize>()
        .map_err(|err| ParseError(format!("Failed to parse Content-Length value: {err}")))
}

// Parse length of the body from the Transfer-Encoding header.
pub(crate) fn parse_transfer_encoding_chunked_length(src: &Bytes, mut offset: usize, content_type: Option<&str>) -> Result<usize, ParseError> {
    let mut total_length: usize = 0;

    // let mut src_bytes: Bytes = src.clone();
    // if content_type == Some("application/json") {
    //     // return Ok(0);
    //     // let mut iter = src.iter();
    //     // while let Some(b) = iter.next() {
    //     //     if *b == b'\\' {
    //     //         iter.next();
    //     //     }
    //     // }
    //     println!("src: {:?}", src.to_vec());
    //     // println!("number of 0x5c: {:?}", &src_bytes.clone().into_iter().filter(|b: &u8| *b == 13).collect());
    //     let filtered_bytes: bytes::Bytes = src_bytes.clone().into_iter().filter(|b: &u8| *b == 13).collect();
    //     println!("number of 0x5c: {:?}", filtered_bytes);
    //     // src_bytes = src_bytes.into_iter().filter(|b| *b != 0x5C).collect();      
    //     // println!("number of 0x5c: {:?}", &src_bytes.into_iter().filter(|b: &u8| *b == 0x5C).count());
    // }

    println!("src: {:?}", src);
    // println!("src_bytes: {:?}", src_bytes);

    loop {
        // Parse the chunk size
        let size_end: usize = src[offset..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| ParseError("Invalid chunk size".to_string()))?
            + offset;
        let size_str: &str = std::str::from_utf8(&src[offset..size_end])
            .map_err(|_| ParseError("Invalid UTF-8 in chunk size".to_string()))?;
        let size: usize = usize::from_str_radix(size_str.trim(), 16)
            .map_err(|_| ParseError("Invalid chunk size".to_string()))?;

        offset = size_end + 2; // Move past the CRLF

        if size == 0 {
            break; // End of chunks
        }

        // Add the size to the total length
        total_length += size;

        offset += size + 2; // Move past the chunk data and CRLF
    }

    Ok(total_length)
}


/// Parses a request or response message body.
///
/// # Arguments
///
/// * `src` - The source bytes.
/// * `range` - The range of the message body in the source bytes.
/// * `content_type` - The value of the Content-Type header.
/// * `transfer_encoding` - The value of the Transfer-Encoding header, if any.
pub(crate) fn parse_body(
    src: &Bytes,
    range: Range<usize>,
    content_type: Option<&str>,
    transfer_encoding: Option<&str>,
) -> Result<Body, ParseError> {

    if transfer_encoding == Some("chunked") {
        let chunks: Vec<Chunk> = parse_chunked_body(src, range.clone())?;
        // println!("chunks: {:?}", chunks);
        // build the chunked body
        let chunked_body: ChunkedBody = build_chunked_body(&chunks);
        return Ok(Body { span: chunked_body.span.clone(), content: BodyContent::Chunked(chunked_body) });
    } 

    let span: Span = Span::new_bytes(src.clone(), range.clone());
    let content: BodyContent = if content_type == Some("application/json") {
        let mut value: json::JsonValue = json::parse(span.data.clone())?;
        value.offset(range.start);

        BodyContent::Json(value)
    } else {
        BodyContent::Unknown(span.clone())
    };

    Ok(Body { span, content })
}

pub(crate) fn parse_chunked_body(src: &Bytes, range: Range<usize>) -> Result<Vec<Chunk>, ParseError> {
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut pos: usize = range.start;

    loop {
        let chunk_size_end: usize = src[pos..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| ParseError("Invalid chunked encoding: missing chunk size CRLF".to_string()))?
            + pos;

        let chunk_size_str = std::str::from_utf8(&src[pos..chunk_size_end])
            .map_err(|_| ParseError("Invalid chunk size encoding".to_string()))?;
        let chunk_size = usize::from_str_radix(chunk_size_str.trim(), 16)
            .map_err(|_| ParseError("Invalid chunk size value".to_string()))?;

        pos = chunk_size_end + 2;

        if chunk_size == 0 {
            break;
        }

        let chunk_data_end = pos + chunk_size;
        if chunk_data_end > src.len() {
            return Err(ParseError("Chunk data exceeds source length".to_string()));
        }
        chunks.push(Chunk {
            span: Span::new_bytes(src.clone(), pos..chunk_data_end),
            data: Bytes::from(src[pos..chunk_data_end].to_vec()),
            extension: None,
        });

        pos = chunk_data_end + 2;
    }

    Ok(chunks)
}
