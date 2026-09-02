// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
// multipart/form-data — the shape a browser posts a file in.
//
// This works on bytes and never on a String, because the whole point of an
// upload is that it may not be text. A PDF pushed through String::from_utf8
// comes out with every byte the decoder disliked replaced by U+FFFD, which is
// a corrupted file that still looks like a successful request.

/// One part of a multipart body. `filename` is what separates a file from an
/// ordinary form field: the format says a part carrying a file names it.
#[derive(Debug, Clone)]
pub struct Part {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

/// Where does `needle` first appear in `haystack`?
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// The boundary named in a Content-Type header, when it is a multipart one.
///
/// `multipart/form-data; boundary=----WebKitFormBoundaryABC`
pub fn boundary_of(content_type: &str) -> Option<String> {
    if !content_type
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("multipart/form-data")
    {
        return None;
    }

    content_type.split(';').skip(1).find_map(|piece| {
        let (name, value) = piece.split_once('=')?;
        if name.trim().eq_ignore_ascii_case("boundary") {
            let value = value.trim().trim_matches('"');
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        } else {
            None
        }
    })
}

/// One parameter out of a header line: `name="value"` or `name=value`.
fn parameter(header: &str, wanted: &str) -> Option<String> {
    header.split(';').skip(1).find_map(|piece| {
        let (name, value) = piece.split_once('=')?;
        if name.trim().eq_ignore_ascii_case(wanted) {
            Some(value.trim().trim_matches('"').to_string())
        } else {
            None
        }
    })
}

/// Split a multipart body into its parts.
///
/// A malformed body yields the parts that could be read rather than an error:
/// the alternative is refusing a whole upload because a trailing byte was
/// wrong, and every part here is independently well-formed or absent.
pub fn parse(body: &[u8], boundary: &str) -> Vec<Part> {
    let delimiter = format!("\r\n--{}", boundary).into_bytes();

    // Every delimiter but the first is preceded by CRLF. Lending the body a
    // leading CRLF makes the first one look like all the others.
    let mut framed = Vec::with_capacity(body.len() + 2);
    framed.extend_from_slice(b"\r\n");
    framed.extend_from_slice(body);

    // Where each delimiter starts.
    let mut marks = Vec::new();
    let mut cursor = 0;
    while let Some(at) = find(&framed[cursor..], &delimiter) {
        marks.push(cursor + at);
        cursor = cursor + at + delimiter.len();
    }

    let mut parts = Vec::new();
    for pair in marks.windows(2) {
        let begin = pair[0] + delimiter.len();
        let end = pair[1];

        // `--` after a delimiter closes the body; nothing after it is a part.
        if framed[begin..].starts_with(b"--") {
            break;
        }

        let segment = &framed[begin..end];
        let segment = match segment.strip_prefix(b"\r\n") {
            Some(rest) => rest,
            None => continue,
        };

        let Some(gap) = find(segment, b"\r\n\r\n") else {
            continue;
        };
        let head = String::from_utf8_lossy(&segment[..gap]);
        let data = segment[gap + 4..].to_vec();

        let mut name = None;
        let mut filename = None;
        let mut content_type = None;
        for line in head.lines() {
            let Some((header, value)) = line.split_once(':') else {
                continue;
            };
            match header.trim().to_ascii_lowercase().as_str() {
                "content-disposition" => {
                    name = parameter(value, "name");
                    filename = parameter(value, "filename");
                }
                "content-type" => content_type = Some(value.trim().to_string()),
                _ => {}
            }
        }

        // A part with no name is not addressable, so it is not a part.
        if let Some(name) = name {
            parts.push(Part {
                name,
                filename,
                content_type,
                data,
            });
        }
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(boundary: &str) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        out.extend_from_slice(b"Content-Disposition: form-data; name=\"title\"\r\n\r\n");
        out.extend_from_slice("ஒப்பந்தம்".as_bytes());
        out.extend_from_slice(format!("\r\n--{}\r\n", boundary).as_bytes());
        out.extend_from_slice(
            b"Content-Disposition: form-data; name=\"doc\"; filename=\"a.pdf\"\r\n",
        );
        out.extend_from_slice(b"Content-Type: application/pdf\r\n\r\n");
        // Deliberately not UTF-8.
        out.extend_from_slice(&[0x25, 0x50, 0x44, 0x46, 0xFF, 0xFE, 0x00, 0x0A]);
        out.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
        out
    }

    #[test]
    fn a_boundary_is_read_from_the_content_type() {
        assert_eq!(
            boundary_of("multipart/form-data; boundary=----abc"),
            Some("----abc".to_string())
        );
        assert_eq!(
            boundary_of("multipart/form-data; boundary=\"quoted\""),
            Some("quoted".to_string())
        );
        assert_eq!(boundary_of("application/json"), None);
    }

    #[test]
    fn fields_and_files_are_told_apart_by_the_filename() {
        let parts = parse(&body("Xbound"), "Xbound");

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name, "title");
        assert!(parts[0].filename.is_none(), "a field has no filename");
        assert_eq!(String::from_utf8_lossy(&parts[0].data), "ஒப்பந்தம்");

        assert_eq!(parts[1].name, "doc");
        assert_eq!(parts[1].filename.as_deref(), Some("a.pdf"));
        assert_eq!(parts[1].content_type.as_deref(), Some("application/pdf"));
    }

    // The reason this module works on bytes at all.
    #[test]
    fn a_file_that_is_not_text_arrives_unchanged() {
        let parts = parse(&body("Xbound"), "Xbound");

        assert_eq!(
            parts[1].data,
            vec![0x25, 0x50, 0x44, 0x46, 0xFF, 0xFE, 0x00, 0x0A]
        );
    }

    #[test]
    fn the_closing_delimiter_does_not_become_a_part() {
        let parts = parse(&body("Xbound"), "Xbound");

        assert!(
            parts.iter().all(|p| !p.name.is_empty()),
            "nothing after the closing -- is read as a part"
        );
    }
}
