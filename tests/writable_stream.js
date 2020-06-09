export function new_noop_writable_stream() {
    return new WritableStream();
}

export function new_recording_writable_stream() {
    const events = [];
    const stream = new WritableStream({
        write(chunk) {
            events.push("write", chunk);
        },
        close() {
            events.push("close");
        },
        abort(e) {
            events.push("abort", e);
        }
    });
    return {stream, events};
};
