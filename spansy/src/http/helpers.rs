use crate::ParseError;
use crate::http::types::{Header, Request, Response};

/// Gets the values of a header in a Request as a list of strings.
pub(crate) fn get_header_values_request<'a>(
    request: &'a Request,
    header_name: &'a str,
) -> Result<Vec<&'a str>, ParseError> {
    get_header_values_from_iter(header_name, request.headers_with_name(header_name))
}

/// Gets the values of a header in a Response as a list of strings.
pub(crate) fn get_header_values_response<'a>(
    response: &'a Response,
    header_name: &'a str,
) -> Result<Vec<&'a str>, ParseError> {
    get_header_values_from_iter(header_name, response.headers_with_name(header_name))
}

/// Gets the values of a header field from an Iterator as a list of strings.
/// Returns a ParseError if any of the values are not valid UTF-8.
pub(crate) fn get_header_values_from_iter<'a>(
    header_name: &'a str,
    headers: impl Iterator<Item = &'a Header>,
) -> Result<Vec<&'a str>, ParseError> {
    headers
        .map(|h| {
            std::str::from_utf8(h.value.0.as_bytes())
                .map(|v| v.trim())
                .map_err(|err| {
                    ParseError(format!(
                        "Invalid UTF-8 when parsing {header_name} header value: {err}"
                    ))
                })
        })
        .collect()
}

pub(crate) fn get_content_type_request(request: &Request) -> Option<&str> {
    match get_header_values_request(&request, "Content-Type") {
        Ok(content_types) if !content_types.is_empty() => Some(content_types[0]),
        _ => None,
    }
}

/// Gets the value of the Transfer-Encoding header in a Request as a string.
/// Returns None if the header is not present or the value is not a valid transfer encoding.
pub(crate) fn get_transfer_encoding_request(request: &Request) -> Option<&str> {
    match get_header_values_request(&request, "Transfer-Encoding") {
        Ok(transfer_encodings) => super::span::ACCEPTED_TRANSFER_ENCODINGS
            .iter()
            .find(|&&accepted| transfer_encodings.contains(&accepted))
            .map(|&v| v),
        _ => None,
    }
}

pub(crate) fn get_content_type_response(response: &Response) -> Option<&str> {
    match get_header_values_response(&response, "Content-Type") {
        Ok(content_types) if !content_types.is_empty() => Some(content_types[0]),
        _ => None,
    }
}

pub(crate) fn get_transfer_encoding_response(response: &Response) -> Option<&str> {
    match get_header_values_response(&response, "Transfer-Encoding") {
        Ok(transfer_encodings) => super::span::ACCEPTED_TRANSFER_ENCODINGS
            .iter()
            .find(|&&accepted| transfer_encodings.contains(&accepted))
            .map(|&v| v),
        _ => None,
    }
}

pub(crate) fn is_valid_transfer_encoding_request(request: &Request) -> bool {
    let transfer_encodings: Vec<&str> = get_header_values_request(request, "Transfer-Encoding").unwrap_or_default();
    transfer_encodings.len() == 0 || transfer_encodings.iter().any(|v| super::span::ACCEPTED_TRANSFER_ENCODINGS.contains(&v))
}

pub(crate) fn is_valid_transfer_encoding_response(response: &Response) -> bool {
    let transfer_encodings: Vec<&str> = get_header_values_response(response, "Transfer-Encoding").unwrap_or_default();
    transfer_encodings.len() == 0 || transfer_encodings.iter().any(|v| super::span::ACCEPTED_TRANSFER_ENCODINGS.contains(&v))
}

pub(crate) fn invalid_transfer_encoding_message_request(request: &Request) -> String {
    let transfer_encodings: Vec<&str> = get_header_values_request(request, "Transfer-Encoding").unwrap_or_default();
    let bad_values: String = transfer_encodings.join(", ");
    format!("Transfer-Encoding other than {} not supported yet: {bad_values}", super::span::ACCEPTED_TRANSFER_ENCODINGS.join(", "))
}

pub(crate) fn invalid_transfer_encoding_message_response(response: &Response) -> String {
    let transfer_encodings: Vec<&str> = get_header_values_response(response, "Transfer-Encoding").unwrap_or_default();
    let bad_values: String = transfer_encodings.join(", ");
    format!("Transfer-Encoding other than {} not supported yet: {bad_values}", super::span::ACCEPTED_TRANSFER_ENCODINGS.join(", "))
}
