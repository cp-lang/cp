trait MutableWorker {
    fn step(mut _);
}

class Worker implements MutableWorker {
    count: i32;
    fn init() {
        _.count = 0;
    }
    fn step(mut _) {
        _.count = _.count + 1;
    }
}

// Parameter `w` is a read-only borrow of `MutableWorker` (missing `mut w`)
fn runWorkerReadOnly(w: MutableWorker) {
    // ERROR: cannot call `mut _` method `step()` on read-only parameter `w`!
    w.step();
}

fn main() {
    let w = new Worker();
    runWorkerReadOnly(w);
}
