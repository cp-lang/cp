import { test, expect_eq } from "cap:test";

class Buffer<const N: usize> {
    pub data: [u8; N];
}

class Tensor<T, const shape: [usize; 2]> {
    pub const rows: usize = shape[0];
    pub const cols: usize = shape[1];
    pub data: [T; shape[0] * shape[1]];
}

type Embedding = Tensor<f32, [1, 4096]>;
type PacketBuffer = Buffer<1024>;

fn main(): void {
    test("Const Generics: Buffer<N>", () => {
        let buf: PacketBuffer = PacketBuffer{ data: [0; 1024] };
        expect_eq(buf.data.len, 1024);
        buf.data[0] = 42;
        expect_eq(buf.data[0], 42);
    });

    test("Const Generics: Tensor<T, [R, C]>", () => {
        let t: Embedding = Embedding{ data: [1.5; 4096] };
        expect_eq(t.data.len, 4096);
        expect_eq(t.data[0], 1.5);
        expect_eq(t.data[4095], 1.5);
        expect_eq(Embedding.rows, 1);
        expect_eq(Embedding.cols, 4096);
    });
}
