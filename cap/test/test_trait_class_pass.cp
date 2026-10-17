import { print } from "cap:io";

class Tensor {
    data: i32;
    fn init(v: i32) {
        _.data = v;
    }
    fn get(): i32 {
        return _.data;
    }
    fn set(mut _, v: i32) {
        _.data = v;
    }
    fn release() {
        @print("Tensor released: {d}", _.data);
    }
}

// 1. Trait with default implementation and `mut _` contract
trait Resettable {
    fn reset(mut _);
    fn resetAndGetCode(mut _): i32 {
        // Omitted `_.` call to required method inside trait default implementation
        reset();
        return 200;
    }
}

trait Inspectable {
    fn inspect(): i32;
}

// 2. Base class with `_` receiver, omitted `_.` calls, and `implements`
class BaseEngine implements Resettable {
    power: i32;
    weights: Tensor;

    fn init(p: i32, move w: Tensor) {
        _.power = p;
        _.weights = w;
    }

    // Default read-only borrow receiver (omitted `_`)
    fn basePower(): i32 {
        return _.power;
    }

    // Sibling call omitting `_.` and using `match` with `*` wildcard (no collision with `_`!)
    fn classifyPower(): i32 {
        let p = basePower();
        let tier = 0;
        match (p) {
            0 => {
                tier = 0;
            },
            100 => {
                tier = 1;
            },
            * => {
                // Inside `*` arm, `_` still refers to `this`!
                tier = _.power + 1;
            }
        }
        return tier;
    }

    // Tier 1 Method Return Lifetime Elision: `&Tensor` automatically binds to receiver `_`
    fn getWeights(): &Tensor {
        return &_.weights;
    }

    // Mutable borrow receiver (`mut _`)
    fn boost(mut _, delta: i32) {
        _.power = _.power + delta;
    }

    fn reset(mut _) {
        _.power = 0;
    }

    fn release() {
        @print("BaseEngine released, power={d}", _.power);
    }
}

// 3. Single Inheritance (`extends`) + `super` calls
class TurboEngine extends BaseEngine {
    turbo_mult: i32;

    fn init(p: i32, move w: Tensor, mult: i32) {
        _.power = p;
        _.weights = w;
        _.turbo_mult = mult;
    }

    // Override mutable method and delegate to `super.boost`
    fn boost(mut _, delta: i32) {
        super.boost(delta * _.turbo_mult);
    }

    // Consuming ownership method (`move _`): consumes receiver and returns final score
    fn shutdownAndExtract(move _): i32 {
        let final_score = _.power + _.weights.get();
        @print("TurboEngine shutting down with score={d}", final_score);
        return final_score;
    }

    fn release() {
        _.weights.release();
        @print("TurboEngine released, power={d}", _.power);
    }
}

// 4. Retroactive trait implementation (`impl Trait for Class`)
impl Inspectable for TurboEngine {
    fn inspect(): i32 {
        return _.power * _.turbo_mult;
    }
}

// 5. Static polymorphism functions with read-only vs `mut` trait parameters
fn readInspectable(item: Inspectable): i32 {
    return item.inspect();
}

fn triggerReset(mut item: Resettable): i32 {
    return item.resetAndGetCode();
}

fn runTest(): i32 {
    let t = new Tensor(42);
    let engine = new TurboEngine(50, t, 3);

    // Omitted `_.` & `match` with `*` wildcard inherited from BaseEngine
    let tier = engine.classifyPower();
    @print("Classified tier: {d}", tier);

    // Tier 1 Method Return Borrow + NLL:
    // `w_ref` borrows `engine` immutably until its last use
    let w_ref = engine.getWeights();
    @print("Borrowed weights: {d}", w_ref.get());
    // `w_ref` NLL lifetime ends here! Now we can call `mut _` methods on `engine`:
    engine.boost(10);
    @print("After turbo boost(10*3): {d}", engine.basePower());

    // Retroactive trait call via free function
    let inspected = readInspectable(engine);
    @print("Inspected value: {d}", inspected);

    // Default trait method call via `mut` trait parameter
    let code = triggerReset(engine);
    @print("Reset code: {d}, power after reset: {d}", code, engine.basePower());

    // Boost once more before consuming
    engine.boost(5);

    // `move _` method consumes `engine` (ownership transferred into `shutdownAndExtract`, released once inside it)
    let final_score = engine.shutdownAndExtract();
    @print("Final extracted score: {d}", final_score);
    return final_score;
}

fn main() {
    let res = runTest();
    @print("Trait & Class Ownership Test Complete: {d}", res);
}
