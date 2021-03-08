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
    let mut buf = [0u8; 3];
    assert_eq!(reader.read(&mut buf).await.unwrap(), 3);
    assert_eq!(&buf, &[1, 2, 3]);
    assert_eq!(reader.read(&mut buf).await.unwrap(), 3);
    assert_eq!(&buf, &[4, 5, 6]);
    assert_eq!(reader.read(&mut buf).await.unwrap(), 0);
    assert_eq!(&buf, &[4, 5, 6]);
    reader.closed().await.unwrap();
}
