import {
    print,
    IOError,
    SeekOrigin,
    Reader,
    Writer,
    BufferReader,
    BufferWriter,
    BufferedReader,
    BufferedWriter,
    copyStream,
    copyN,
    MaxBuf,
    Pipe,
} from "cap:io";

fn expect(cond: bool, msg: string): void {
    if (!cond) {
        print(`ASSERTION FAILED: ${msg}`);
        @panic("assertion failed");
    }
}

pub fn main(): void {
    print("=== Running Industrial-grade I/O Stream Suite (cap:io / cap:stream) ===");

    // Test 1: BufferReader slicing and byte reads
    let source_text = "The quick brown fox jumps over the lazy dog.";
    let reader = new BufferReader(source_text);
    expect(reader.remaining() == source_text.len, "BufferReader initial remaining bytes");

    let chunk: [16]u8 = [0 as u8; 16];
    let n1 = reader.read(chunk[0..16]) catch 0;
    expect(n1 == 16, "BufferReader read first 16 bytes");
    expect(reader.remaining() == source_text.len - 16, "BufferReader remaining after 16 bytes");

    // Test 2: BufferReader seeking
    let seek_pos = reader.seek(0, SeekOrigin.Start) catch 999;
    expect(seek_pos == 0, "BufferReader seek to start");
    expect(reader.remaining() == source_text.len, "BufferReader remaining after seek to start");

    let seek_curr = reader.seek(4, SeekOrigin.Current) catch 999;
    expect(seek_curr == 4, "BufferReader seek relative +4");

    // Test 3: BufferWriter and copyStream pipeline
    reader.seek(0, SeekOrigin.Start) catch {};
    let writer = new BufferWriter();
    let copied = copyStream(reader, writer) catch 0;
    expect(copied == source_text.len, "copyStream transferred full buffer");
    expect(writer.len() == source_text.len, "writer length matches source text length");

    let out_slice = writer.bytes();
    expect(out_slice.len == source_text.len, "writer bytes() length matches");

    // Test 4: copyN partial copy
    reader.seek(0, SeekOrigin.Start) catch {};
    let writer2 = new BufferWriter();
    let copied_n = copyN(reader, writer2, 10) catch 0;
    expect(copied_n == 10, "copyN copied exactly 10 bytes");
    expect(writer2.len() == 10, "writer2 length is 10");

    // Test 5: BufferedReader byte-by-byte caching & fast multi-byte reads
    let stream_data = "ABCDEFGH\n12345678\nBuffered I/O in CP is zero-copy and blazing fast!";
    let base_r = new BufferReader(stream_data);
    let buf_r = new BufferedReader<BufferReader>(base_r);

    let b0: u8 = buf_r.readByte() catch 0;
    expect(b0 == 65, "readByte returned 'A' (65)");
    let b1: u8 = buf_r.readByte() catch 0;
    expect(b1 == 66, "readByte returned 'B' (66)");

    let buf_chunk: [16]u8 = [0 as u8; 16];
    let read_cnt = buf_r.read(buf_chunk[0..16]) catch 0;
    expect(read_cnt == 16, "BufferedReader block read 16 bytes");

    // Test 6: BufferedWriter caching and flush
    let base_w = new BufferWriter();
    let buf_w = new BufferedWriter<BufferWriter>(base_w);
    let data_to_write = "Pipeline stream output via BufferedWriter.";
    let w_cnt = buf_w.write(data_to_write) catch 0;
    expect(w_cnt == data_to_write.len, "BufferedWriter write returned full count");
    buf_w.flush() catch {};
    expect(base_w.len() == data_to_write.len, "Underlying BufferWriter received flushed data");

    // Test 7: MaxBuf boundary guard (from Bun.js)
    let max_buf = new MaxBuf(10);
    let ok1: bool = max_buf.onReadBytes(6) catch false;
    expect(ok1 == true, "MaxBuf onReadBytes within limit");
    let ok2: bool = max_buf.onReadBytes(4) catch false;
    expect(ok2 == true, "MaxBuf onReadBytes reached limit");
    let ok3: bool = max_buf.onReadBytes(1) catch false;
    expect(ok3 == false, "MaxBuf onReadBytes exceeded limit triggers error");

    // Test 8: PipeReader and PipeWriter IPC stream
    let pipe = Pipe.create() catch unreachable;
    let msg = "Hello from PipeWriter to PipeReader!";
    let zero_sz: usize = 0;
    let p_written: usize = pipe.writer.write(msg) catch zero_sz;
    expect(p_written == msg.len, "PipeWriter written length matches message");

    let p_buf: [64]u8 = [0 as u8; 64];
    let p_read: usize = pipe.reader.read(p_buf[0..64]) catch zero_sz;
    expect(p_read == msg.len, "PipeReader read exact bytes written by PipeWriter");
    expect(p_buf[0] == 72, "First byte is 'H' (72)");

    pipe.reader.close();
    pipe.writer.close();

    print("=== All 8 Industrial I/O Stream Test Suites PASSED Successfully! ===");
}
