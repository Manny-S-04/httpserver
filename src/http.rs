pub const MAX_HEAD: usize = 8 * 1024;
pub const MAX_BODY: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    BadRequestLine,
    BadVersion,
    BadHeader,
    MissingHost,
    BadContentLength,
    Unsupported,
    HeadTooLarge,
    BodyTooLarge,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Parse {
    Incomplete,
    Done(Request, usize),
    Error(ParseError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

pub fn find_head_end(buf: &[u8]) -> Option<usize> {
    for i in 0..buf.len().saturating_sub(3) {
        if &buf[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

fn is_tchar(c: u8) -> bool {
    c.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&c)
}

fn is_token(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(is_tchar)
}

pub fn parse_request_line(line: &str) -> Result<(String, String, String), ParseError> {
    let parts: Vec<&str> = line.split(' ').collect();
    if parts.len() != 3 || !is_token(parts[0]) {
        return Err(ParseError::BadRequestLine);
    }

    let path = parts[1];
    if path.is_empty() || !(path.starts_with('/') || path == "*") {
        return Err(ParseError::BadRequestLine);
    }
    for c in path.bytes() {
        if c <= b' ' || c == 0x7f {
            return Err(ParseError::BadRequestLine);
        }
    }

    let version = parts[2];
    if version != "HTTP/1.1" && version != "HTTP/1.0" {
        return Err(ParseError::BadVersion);
    }

    Ok((parts[0].to_string(), path.to_string(), version.to_string()))
}

pub fn parse_header(line: &str) -> Result<Header, ParseError> {
    let Some(colon) = line.find(':') else {
        return Err(ParseError::BadHeader);
    };
    let name = &line[..colon];
    // rfc 9112 5.1, 5.2
    if !is_token(name) {
        return Err(ParseError::BadHeader);
    }
    let value = line[colon + 1..].trim_matches([' ', '\t']);
    for c in value.bytes() {
        if (c < b' ' && c != b'\t') || c == 0x7f {
            return Err(ParseError::BadHeader);
        }
    }
    Ok(Header {
        name: name.to_string(),
        value: value.to_string(),
    })
}

pub fn header<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    for h in &req.headers {
        if h.name.eq_ignore_ascii_case(name) {
            return Some(&h.value);
        }
    }
    None
}

pub fn content_length(headers: &[Header]) -> Result<usize, ParseError> {
    let mut length: Option<usize> = None;
    for h in headers {
        if !h.name.eq_ignore_ascii_case("content-length") {
            continue;
        }
        if h.value.is_empty() || !h.value.bytes().all(|c| c.is_ascii_digit()) {
            return Err(ParseError::BadContentLength);
        }
        let Ok(n) = h.value.parse::<usize>() else {
            return Err(ParseError::BadContentLength);
        };
        // smuggling
        if length.is_some() && length != Some(n) {
            return Err(ParseError::BadContentLength);
        }
        length = Some(n);
    }
    Ok(length.unwrap_or(0))
}

pub fn parse_head(head: &[u8]) -> Result<Request, ParseError> {
    let Ok(text) = std::str::from_utf8(head) else {
        return Err(ParseError::BadRequestLine);
    };

    let mut lines = text.split("\r\n");
    let (method, path, version) = parse_request_line(lines.next().unwrap_or(""))?;

    let mut headers: Vec<Header> = Vec::new();
    for line in lines {
        headers.push(parse_header(line)?);
    }

    let req = Request {
        method,
        path,
        version,
        headers,
        body: Vec::new(),
    };

    if req.version == "HTTP/1.1" && header(&req, "host").is_none() {
        return Err(ParseError::MissingHost);
    }
    if header(&req, "transfer-encoding").is_some() {
        return Err(ParseError::Unsupported);
    }
    Ok(req)
}

pub fn parse_request(buf: &[u8]) -> Parse {
    let Some(end) = find_head_end(buf) else {
        if buf.len() > MAX_HEAD {
            return Parse::Error(ParseError::HeadTooLarge);
        }
        return Parse::Incomplete;
    };
    if end > MAX_HEAD {
        return Parse::Error(ParseError::HeadTooLarge);
    }

    let mut req = match parse_head(&buf[..end]) {
        Ok(req) => req,
        Err(e) => return Parse::Error(e),
    };
    let length = match content_length(&req.headers) {
        Ok(n) => n,
        Err(e) => return Parse::Error(e),
    };
    if length > MAX_BODY {
        return Parse::Error(ParseError::BodyTooLarge);
    }

    let body_start = end + 4;
    if buf.len() < body_start + length {
        return Parse::Incomplete;
    }
    req.body = buf[body_start..body_start + length].to_vec();
    Parse::Done(req, body_start + length)
}

pub const fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Content Too Large",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        505 => "HTTP Version Not Supported",
        _ => "Unknown",
    }
}

pub const fn error_status(e: ParseError) -> u16 {
    match e {
        ParseError::BadRequestLine => 400,
        ParseError::BadVersion => 505,
        ParseError::BadHeader => 400,
        ParseError::MissingHost => 400,
        ParseError::BadContentLength => 400,
        ParseError::Unsupported => 501,
        ParseError::HeadTooLarge => 431,
        ParseError::BodyTooLarge => 413,
    }
}

pub fn text(status: u16, body: &str) -> Response {
    Response {
        status,
        content_type: "text/plain; charset=utf-8",
        body: body.as_bytes().to_vec(),
    }
}

pub fn status_page(status: u16) -> Response {
    text(status, &format!("{} {}\n", status, reason(status)))
}

pub fn response_bytes(res: &Response, head_only: bool) -> Vec<u8> {
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        res.status,
        reason(res.status),
        res.content_type,
        res.body.len()
    );
    let mut out = head.into_bytes();
    if !head_only {
        out.extend_from_slice(&res.body);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const GET: &[u8] = b"GET /index.html HTTP/1.1\r\nHost: localhost\r\nUser-Agent: test\r\n\r\n";

    fn done(buf: &[u8]) -> (Request, usize) {
        match parse_request(buf) {
            Parse::Done(req, used) => (req, used),
            other => panic!("expected Done, got {other:?}"),
        }
    }

    fn error(buf: &[u8]) -> ParseError {
        match parse_request(buf) {
            Parse::Error(e) => e,
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn request_line() {
        let (req, used) = done(GET);
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/index.html");
        assert_eq!(req.version, "HTTP/1.1");
        assert_eq!(used, GET.len());
    }

    #[test]
    fn headers() {
        let (req, _) = done(GET);
        assert_eq!(req.headers.len(), 2);
        assert_eq!(header(&req, "HOST"), Some("localhost"));
        assert_eq!(header(&req, "user-agent"), Some("test"));
        assert_eq!(header(&req, "accept"), None);
    }

    #[test]
    fn split_across_reads() {
        for i in 0..GET.len() {
            assert_eq!(parse_request(&GET[..i]), Parse::Incomplete, "prefix {i}");
        }
        done(GET);
    }

    #[test]
    fn body_split_across_reads() {
        let raw = b"POST /x HTTP/1.1\r\nHost: a\r\nContent-Length: 5\r\n\r\nhello";
        for i in 0..raw.len() {
            assert_eq!(parse_request(&raw[..i]), Parse::Incomplete, "prefix {i}");
        }
        let (req, used) = done(raw);
        assert_eq!(req.body, b"hello");
        assert_eq!(used, raw.len());
    }

    #[test]
    fn pipelined_leaves_rest() {
        let mut raw = GET.to_vec();
        raw.extend_from_slice(b"GET /next");
        let (_, used) = done(&raw);
        assert_eq!(&raw[used..], b"GET /next");
    }

    #[test]
    fn value_whitespace_trimmed() {
        let (req, _) = done(b"GET / HTTP/1.1\r\nHost:   a b \t\r\n\r\n");
        assert_eq!(header(&req, "host"), Some("a b"));
    }

    #[test]
    fn http10_no_host() {
        let (req, _) = done(b"GET / HTTP/1.0\r\n\r\n");
        assert_eq!(req.version, "HTTP/1.0");
    }

    #[test]
    fn bad_request_lines() {
        assert_eq!(
            error(b"GET /\r\nHost: a\r\n\r\n"),
            ParseError::BadRequestLine
        );
        assert_eq!(
            error(b"GET  / HTTP/1.1\r\nHost: a\r\n\r\n"),
            ParseError::BadRequestLine
        );
        assert_eq!(
            error(b"GET index HTTP/1.1\r\nHost: a\r\n\r\n"),
            ParseError::BadRequestLine
        );
        assert_eq!(
            error(b"G(T / HTTP/1.1\r\nHost: a\r\n\r\n"),
            ParseError::BadRequestLine
        );
        assert_eq!(
            error(b"GET / HTTP/2.0\r\nHost: a\r\n\r\n"),
            ParseError::BadVersion
        );
        assert_eq!(
            error(b"\xff / HTTP/1.1\r\n\r\n"),
            ParseError::BadRequestLine
        );
    }

    #[test]
    fn bad_headers() {
        assert_eq!(
            error(b"GET / HTTP/1.1\r\nHost : a\r\n\r\n"),
            ParseError::BadHeader
        );
        assert_eq!(
            error(b"GET / HTTP/1.1\r\nHost: a\r\n folded\r\n\r\n"),
            ParseError::BadHeader
        );
        assert_eq!(
            error(b"GET / HTTP/1.1\r\nnocolon\r\n\r\n"),
            ParseError::BadHeader
        );
        assert_eq!(error(b"GET / HTTP/1.1\r\n\r\n"), ParseError::MissingHost);
    }

    #[test]
    fn content_length_rules() {
        let two = b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\nab";
        assert_eq!(error(two), ParseError::BadContentLength);
        let signed = b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: +1\r\n\r\na";
        assert_eq!(error(signed), ParseError::BadContentLength);
        let same = b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\na";
        assert_eq!(done(same).0.body, b"a");
        let chunked = b"POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n";
        assert_eq!(error(chunked), ParseError::Unsupported);
    }

    #[test]
    fn too_large() {
        let mut big = b"GET / HTTP/1.1\r\nHost: a\r\nX: ".to_vec();
        big.resize(MAX_HEAD + 10, b'a');
        assert_eq!(error(&big), ParseError::HeadTooLarge);
        let huge = format!(
            "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: {}\r\n\r\n",
            MAX_BODY + 1
        );
        assert_eq!(error(huge.as_bytes()), ParseError::BodyTooLarge);
    }

    #[test]
    fn response() {
        let out = response_bytes(&text(200, "hi"), false);
        assert_eq!(
            out,
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 2\r\nConnection: close\r\n\r\nhi"
        );
        let head = response_bytes(&text(200, "hi"), true);
        assert!(head.ends_with(b"Content-Length: 2\r\nConnection: close\r\n\r\n"));
    }
}
