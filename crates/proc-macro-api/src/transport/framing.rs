//! Protocol framing

use std::io::{self, BufRead, Write};

pub trait Framing {
    type Buf: Default + Send + Sync;

    fn read<'a, R: BufRead + ?Sized>(
        inp: &mut R,
        buf: &'a mut Self::Buf,
    ) -> io::Result<Option<&'a mut Self::Buf>>;

    fn write<W: Write + ?Sized>(out: &mut W, buf: &Self::Buf) -> io::Result<()>;
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use crate::transport::codec::json::JsonProtocol;
    use crate::transport::codec::postcard::PostcardProtocol;
    use crate::transport::framing::Framing;

    // ========================================================================
    // JSON Framing Tests
    // ========================================================================

    mod json_framing {
        use super::*;

        #[test]
        fn test_json_framing_read_single_line() {
            let data = "{\"field\": \"value\"}\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"field\": \"value\"}");
        }

        #[test]
        fn test_json_framing_read_multiple_lines() {
            let data = "{\"msg\": 1}\n{\"msg\": 2}\n{\"msg\": 3}\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            // First message
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"msg\": 1}");

            // Second message
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"msg\": 2}");

            // Third message
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"msg\": 3}");

            // EOF
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn test_json_framing_read_empty_input() {
            let data = "";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn test_json_framing_ignores_non_json_output() {
            // Proc-macros might print debug output before JSON
            let data = "Some debug output\nAnother line of garbage\n{\"valid\": true}\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            // Should skip non-JSON lines and return the valid JSON
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"valid\": true}");
        }

        #[test]
        fn test_json_framing_ignores_multiple_non_json_lines() {
            let data =
                "debug: starting\nwarning: something\nerror output\n{\"data\": 42}\nmore garbage\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"data\": 42}");
        }

        #[test]
        fn test_json_framing_write_adds_newline() {
            let msg = "{\"test\": true}".to_string();
            let mut output = Vec::new();

            JsonProtocol::write(&mut output, &msg).unwrap();

            assert_eq!(output, b"{\"test\": true}\n");
        }

        #[test]
        fn test_json_framing_write_multiple_messages() {
            let mut output = Vec::new();

            JsonProtocol::write(&mut output, &"{\"msg\": 1}".to_string()).unwrap();
            JsonProtocol::write(&mut output, &"{\"msg\": 2}".to_string()).unwrap();
            JsonProtocol::write(&mut output, &"{\"msg\": 3}".to_string()).unwrap();

            assert_eq!(output, b"{\"msg\": 1}\n{\"msg\": 2}\n{\"msg\": 3}\n");
        }

        #[test]
        fn test_json_framing_roundtrip() {
            let original = "{\"roundtrip\": \"test\"}".to_string();

            // Write
            let mut output = Vec::new();
            JsonProtocol::write(&mut output, &original).unwrap();

            // Read back
            let mut reader = BufReader::new(Cursor::new(output));
            let mut buf = String::new();
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();

            assert!(result.is_some());
            assert_eq!(result.unwrap(), &original);
        }

        #[test]
        fn test_json_framing_empty_json_object() {
            let data = "{}\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{}");
        }

        #[test]
        fn test_json_framing_json_array() {
            // Arrays don't start with '{', so they should be filtered out
            let data = "[1, 2, 3]\n{\"valid\": true}\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            // Should skip the array and return the object
            assert_eq!(result.unwrap(), "{\"valid\": true}");
        }

        #[test]
        fn test_json_framing_clears_buffer_between_reads() {
            let data = "{\"first\": 1}\n{\"second\": 2}\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            // Read first message
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"first\": 1}");

            // Buffer should be cleared and contain only second message
            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "{\"second\": 2}");
            // Verify buffer doesn't contain first message
            assert!(!buf.contains("first"));
        }

        #[test]
        fn test_json_framing_only_garbage_returns_none() {
            let data = "garbage line 1\ngarbage line 2\n";
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = String::new();

            let result = JsonProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_none());
        }
    }

    // ========================================================================
    // Postcard Framing Tests
    // ========================================================================

    mod postcard_framing {
        use super::*;

        #[test]
        fn test_postcard_framing_read_null_terminated() {
            // Data with null terminator
            let data: Vec<u8> = vec![1, 2, 3, 4, 5, 0];
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = Vec::new();

            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), &vec![1, 2, 3, 4, 5, 0]);
        }

        #[test]
        fn test_postcard_framing_read_multiple_messages() {
            // Multiple null-terminated messages
            let data: Vec<u8> = vec![1, 2, 0, 3, 4, 5, 0, 6, 0];
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = Vec::new();

            // First message
            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), &vec![1, 2, 0]);

            // Second message
            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), &vec![3, 4, 5, 0]);

            // Third message
            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), &vec![6, 0]);

            // EOF
            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn test_postcard_framing_read_empty_input() {
            let data: Vec<u8> = vec![];
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = Vec::new();

            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn test_postcard_framing_write() {
            let msg: Vec<u8> = vec![1, 2, 3, 4, 5];
            let mut output = Vec::new();

            PostcardProtocol::write(&mut output, &msg).unwrap();

            // Should write the exact bytes
            assert_eq!(output, vec![1, 2, 3, 4, 5]);
        }

        #[test]
        fn test_postcard_framing_write_with_null() {
            // COBS-encoded data typically ends with null
            let msg: Vec<u8> = vec![1, 2, 3, 0];
            let mut output = Vec::new();

            PostcardProtocol::write(&mut output, &msg).unwrap();

            assert_eq!(output, vec![1, 2, 3, 0]);
        }

        #[test]
        fn test_postcard_framing_roundtrip() {
            let original: Vec<u8> = vec![10, 20, 30, 40, 0];

            // Write
            let mut output = Vec::new();
            PostcardProtocol::write(&mut output, &original).unwrap();

            // Read back
            let mut reader = BufReader::new(Cursor::new(output));
            let mut buf = Vec::new();
            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();

            assert!(result.is_some());
            assert_eq!(result.unwrap(), &original);
        }

        #[test]
        fn test_postcard_framing_single_null_byte() {
            let data: Vec<u8> = vec![0];
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = Vec::new();

            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), &vec![0]);
        }

        #[test]
        fn test_postcard_framing_clears_buffer_between_reads() {
            let data: Vec<u8> = vec![1, 2, 3, 0, 4, 5, 6, 0];
            let mut reader = BufReader::new(Cursor::new(data));
            let mut buf = Vec::new();

            // Read first message
            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), &vec![1, 2, 3, 0]);

            // Read second message - buffer should be cleared
            let result = PostcardProtocol::read(&mut reader, &mut buf).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap(), &vec![4, 5, 6, 0]);
            // Verify buffer only has second message
            assert_eq!(buf.len(), 4);
        }

        #[test]
        fn test_postcard_framing_write_empty() {
            let msg: Vec<u8> = vec![];
            let mut output = Vec::new();

            PostcardProtocol::write(&mut output, &msg).unwrap();

            assert!(output.is_empty());
        }
    }
}
