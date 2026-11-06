export class Socket {
    fd: i32;

    fn init(fd: i32): void {
        this.fd = fd;
    }

    async fn read(): string {
        return await @net_read_async(this.fd);
    }

    async fn write(data: string): i32 {
        return await @net_write_async(this.fd, data);
    }

    fn close(): void {
        @net_close(this.fd);
    }
}

export class Server {
    fd: i32;

    fn init(fd: i32): void {
        this.fd = fd;
    }

    async fn accept(): Socket {
        let fd = await @net_accept_async(this.fd);
        let s = new Socket(fd);
        return s;
    }

    fn close(): void {
        @net_close(this.fd);
    }
}

export fn listen(port: i32): Server {
    let s = new Server(0); // placeholder, wait, @net_listen needs to create fd
    let fd = @net_listen(port);
    s.fd = fd;
    return s;
}
