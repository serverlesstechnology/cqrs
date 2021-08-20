use iron::Headers;

pub fn std_headers() -> Headers {
    let mut headers = Headers::new();
    let content_type = iron::headers::ContentType::json();
    headers.set(content_type);
    headers
}
