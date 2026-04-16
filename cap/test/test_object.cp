interface Logger {
    fn log(this, msg: string): void;
}

class MyLogger implements Logger {
    prefix: string,
    fn log(this, msg: string): void {
        @print("LOG: {s}", msg);
    }
}

fn main(): ?void {
    let logger: Logger = new MyLogger();
    logger.log("Object model works!");
}