import { print } from "cap:io";

fn test_unbounded(*ptr: [*]u8!): void {
    ptr[10] = 99;
}

fn main(): void {
    print("fail\n");
}
