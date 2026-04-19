export class Socket {
    handle: i32,
    fn send(this, data: string): void {
        @net_send(this.handle, data);
    }
    async fn read(this): string {
        return @net_read_async(this.handle);
    }
    fn close(this): void {
        @net_close(this.handle);
    }
}

export fn connect(host: string, port: i32): Socket {
    const handle = @net_connect(host, port);
    const s = new Socket();
    s.handle = handle;
    return s;
}

export fn listen(port: i32, callback: any): void {
    @net_listen(port, callback);
}
