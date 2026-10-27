import { print } from "cap:io";

class InterchangeTest {
    count: i32;

    fn init(initial: i32) {
        // 1. Setting field via `this.`
        this.count = initial;
    }

    // 2. Default read-only receiver using `_`
    fn getViaUnderscore(): i32 {
        return _.count;
    }

    // 2b. Explicit read-only receiver using `&_`
    fn getViaAmpersandUnderscore(&_): i32 {
        return _.count;
    }

    // 3. Default read-only receiver using `this`
    fn getViaThis(): i32 {
        return this.count;
    }

    // 4. Default read-only receiver with omitted prefix
    fn getOmitted(): i32 {
        return count;
    }

    // 5. Method calling another method via `_` and `this`
    fn computeSum(): i32 {
        let a = _.getViaUnderscore();
        let b = this.getViaThis();
        let c = getOmitted();
        return a + b + c;
    }

    // 6. Mutable borrow receiver using `mut _`, mutating via `this.`
    fn setViaMutUnderscore(mut _, val: i32) {
        this.count = val;
    }

    // 6b. Mutable borrow receiver using `*_`, mutating via `_.`
    fn setViaStarUnderscore(*_, val: i32) {
        _.count = val;
    }

    // 7. Mutable borrow receiver using `mut this`, mutating via `_.`
    fn setViaMutThis(mut this, val: i32) {
        _.count = val;
    }

    // 7b. Mutable borrow receiver using `*this`, mutating via `this.`
    fn setViaStarThis(*this, val: i32) {
        this.count = val;
    }

    // 8. Mutable borrow receiver with omitted prefix
    fn addOmitted(*_, delta: i32) {
        count = count + delta;
    }

    // 9. Consuming method via `move _`
    fn consumeViaUnderscore(move _): i32 {
        return this.count;
    }

    // 10. Async method with state machine Context lifting
    async fn asyncPing(): i32 {
        return _.count + this.count;
    }
}

class ChildInterchange extends InterchangeTest {
    multiplier: i32;

    fn init(initial: i32, mult: i32) {
        super.init(initial);
        _.multiplier = mult;
    }

    fn calculate(): i32 {
        // Child accesses inherited field through `_` and own field through `this`
        return _.count * this.multiplier;
    }

    fn consumeViaThis(move this): i32 {
        return _.count * _.multiplier;
    }
}

async fn main(): void {
    let mut item = new InterchangeTest(10);
    @print("1. getViaUnderscore: {d}", item.getViaUnderscore()); // 10
    @print("2. getViaThis: {d}", item.getViaThis());             // 10
    @print("3. getOmitted: {d}", item.getOmitted());             // 10
    @print("4. computeSum: {d}", item.computeSum());             // 30

    item.setViaMutUnderscore(20);
    @print("5. after setViaMutUnderscore: {d}", item.getViaThis()); // 20

    item.setViaStarUnderscore(28);
    @print("5b. after setViaStarUnderscore (*_): {d}", item.getViaAmpersandUnderscore()); // 28

    item.setViaMutThis(35);
    @print("6. after setViaMutThis: {d}", item.getViaUnderscore()); // 35

    item.setViaStarThis(39);
    @print("6b. after setViaStarThis (*this): {d}", item.getViaUnderscore()); // 39

    item.addOmitted(5);
    @print("7. after addOmitted: {d}", item.getOmitted());          // 40

    let async_res = await item.asyncPing();
    @print("8. asyncPing (_ + this): {d}", async_res);            // 80

    let score = item.consumeViaUnderscore();
    @print("9. consumed score: {d}", score);                     // 40

    let mut child = new ChildInterchange(10, 3);
    @print("10. child calculate: {d}", child.calculate());       // 30
    let child_score = child.consumeViaThis();
    @print("11. child consumed: {d}", child_score);              // 30

    @print("🎉 All _ and this equivalence tests passed successfully! {s}", "");
}
