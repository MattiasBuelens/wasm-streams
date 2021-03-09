use futures::io::AsyncReadExt;
use js_sys::Uint8Array;
use wasm_bindgen_test::*;

use wasm_streams::readable::*;

use crate::js::*;

#[wasm_bindgen_test]
async fn test_readable_byte_stream_new() {
    let mut readable = ReadableStream::from_raw(new_readable_byte_stream_from_array(
        vec![
            Uint8Array::from(&[1, 2, 3][..]).into(),
            Uint8Array::from(&[4, 5, 6][..]).into(),
        ]
        .into_boxed_slice(),
    ));
    assert!(!readable.is_locked());

    let mut reader = readable.get_byob_reader();
    let mut dst = [0u8; 3];
    assert_eq!(reader.read(&mut dst).await.unwrap(), 3);
    assert_eq!(&dst, &[1, 2, 3]);
    assert_eq!(reader.read(&mut dst).await.unwrap(), 3);
    assert_eq!(&dst, &[4, 5, 6]);
    assert_eq!(reader.read(&mut dst).await.unwrap(), 0);
    assert_eq!(&dst, &[4, 5, 6]);
    reader.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_byte_stream_read_with_buffer() {
    let mut readable = ReadableStream::from_raw(new_readable_byte_stream_from_array(
        vec![
            Uint8Array::from(&[1, 2, 3][..]).into(),
            Uint8Array::from(&[4, 5, 6][..]).into(),
        ]
        .into_boxed_slice(),
    ));
    assert!(!readable.is_locked());

    let mut reader = readable.get_byob_reader();
    let mut dst = [0u8; 3];
    let buf = Uint8Array::new_with_length(3);
    let (bytes_read, buf) = reader.read_with_buffer(&mut dst, buf).await.unwrap();
    assert_eq!(bytes_read, 3);
    assert_eq!(&dst, &[1, 2, 3]);
    let (bytes_read, buf) = reader.read_with_buffer(&mut dst, buf).await.unwrap();
    assert_eq!(bytes_read, 3);
    assert_eq!(&dst, &[4, 5, 6]);
    let (bytes_read, buf) = reader.read_with_buffer(&mut dst, buf).await.unwrap();
    assert_eq!(bytes_read, 0);
    assert_eq!(&dst, &[4, 5, 6]);
    drop(buf);
    reader.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_byte_stream_into_async_read() {
    let readable = ReadableStream::from_raw(new_readable_byte_stream_from_array(
        vec![
            Uint8Array::from(&[1, 2, 3][..]).into(),
            Uint8Array::from(&[4, 5, 6][..]).into(),
        ]
        .into_boxed_slice(),
    ));
    assert!(!readable.is_locked());

    let mut async_read = readable.into_async_read();
    let mut buf = [0u8; 3];
    assert_eq!(async_read.read(&mut buf).await.unwrap(), 3);
    assert_eq!(&buf, &[1, 2, 3]);
    assert_eq!(async_read.read(&mut buf[..1]).await.unwrap(), 1);
    assert_eq!(&buf, &[4, 2, 3]);
    assert_eq!(async_read.read(&mut buf[1..]).await.unwrap(), 2);
    assert_eq!(&buf, &[4, 5, 6]);
    assert_eq!(async_read.read(&mut buf).await.unwrap(), 0);
    assert_eq!(&buf, &[4, 5, 6]);
}
