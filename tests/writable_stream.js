export function new_noop_writable_stream() {
    return new WritableStream();
}

export function new_logging_writable_stream() {
    return new WritableStream({
        write(chunk) {
            console.log("wrote chunk:", chunk);
        },
        close() {
            console.log("closed");
        }
    });
}