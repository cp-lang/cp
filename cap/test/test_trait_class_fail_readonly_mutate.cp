class Counter {
    val: i32;
    fn init(v: i32) {
        _.val = v;
    }
    // Default read-only borrow receiver (omitted `_`)
    fn badMutate(): i32 {
        _.val = _.val + 1;
        return _.val;
    }
}

fn main() {
    let c = new Counter(10);
    let x = c.badMutate();
}
