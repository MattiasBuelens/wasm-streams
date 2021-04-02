export function new_noop_readable_stream() {
    return new ReadableStream();
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
