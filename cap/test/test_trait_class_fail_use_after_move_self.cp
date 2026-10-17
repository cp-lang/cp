class Connection {
    fd: i32;
    fn init(f: i32) {
        _.fd = f;
    }
    fn ping(): i32 {
        return _.fd;
    }
    fn close(move _) {
        _.fd = 0;
    }
}

fn runFail(): i32 {
    let conn = new Connection(8080);
    conn.close();
    // ERROR: `conn` was consumed by `close(move _)`!
    return conn.ping();
}

fn main() {
    let x = runFail();
}
