import { print } from "cap:io";

trait Printable {
    fn display(): void;
}

class Cat implements Printable {
    name: string;
    fn display(): void {
        print(this.name);
    }
}

fn show<T: Printable>(item: T): void {
    item.display();
}

fn max_val<T: Comparable>(a: T, b: T): T {
    if (a > b) {
        return a;
    }
    return b;
}

fn main(): void {
    let cat = new Cat("Luna");
    show(cat);

    let m1 = max_val(10, 20);
    print(m1);

    let m2 = max_val(3.14, 2.71);
    print(m2);

    print("generic trait bounds test passed!");
}
