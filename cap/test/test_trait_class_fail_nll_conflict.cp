import { print } from "cap:io";

class Tensor {
    data: i32;
    fn init(v: i32) {
        _.data = v;
    }
    fn get(): i32 {
        return _.data;
    }
}

class Model {
    w: Tensor;
    fn init(t: Tensor) {
        _.w = t;
    }
    fn getWeights(): &Tensor {
        return &_.w;
    }
    fn updateWeights(mut _, v: i32) {
        _.w.data = v;
    }
}

fn runFail() {
    let t = new Tensor(10);
    let m = new Model(t);
    let w_ref = m.getWeights();
    // ERROR: `w_ref` is still active on the next line, so calling `mut _` method on `m` conflicts!
    m.updateWeights(99);
    @print("Weights: {d}", w_ref.get());
}

fn main() {
    runFail();
}
