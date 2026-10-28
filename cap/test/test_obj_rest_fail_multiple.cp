fn main() {
    let obj = { a: 1, b: 2, c: 3 };
    let { a, ...b, ...c } = obj;
}
