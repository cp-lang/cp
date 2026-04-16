export fn spawn(target: any): PID {
    return @spawn(target);
}

export fn receive(): any {
    return @receive();
}

export fn reply(target: PID, msg: any): void {
    @reply(target, msg);
}

export async fn join(p1: PID, p2: PID): any {
    return @join(p1, p2);
}
