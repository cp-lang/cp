// test_dependency.cp
import { add } from "std-math/math";
import { print } from "std/io";

fn main(): ?void {
    let result = add(10, 20);
    @print("Result from dependency: {d}", result);
}
