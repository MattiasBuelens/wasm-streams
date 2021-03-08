export function new_noop_readable_stream() {
    return new ReadableStream();
}

export function new_noop_readable_byte_stream() {
    return new ReadableStream({ type: 'bytes' });
}

export function new_readable_stream_from_array(chunks) {
    return new ReadableStream({
        start(controller) {
            for (let chunk of chunks) {
                controller.enqueue(chunk);
            }
            controller.close();
        }
    });
}

export function new_readable_byte_stream_from_array(chunks) {
    return new ReadableStream({
        type: 'bytes',
        start(controller) {
            for (let chunk of chunks) {
                controller.enqueue(chunk);
            }
            controller.close();
        }
    });
}