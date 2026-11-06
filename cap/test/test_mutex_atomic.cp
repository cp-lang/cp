import { test, expect_eq } from "cap:test";
import { Mutex, RwLock, Atomic } from "cap:sync";

struct Counter {
    count: i32,
}

struct State {
    value: i32,
}

fn main(): void {
    test("Mutex<T> basic withLock, lock, tryLock", () => {
        let mu = Mutex(Counter { count: 0 });
        mu.withLock((*s) => {
            s.count = s.count + 10;
        });
        let cur = mu.withLock((*s) => s.count);
        expect_eq(cur, 10);

        let guard = mu.lock();
        guard.count = guard.count + 5;
        guard.release();
        expect_eq(mu.withLock((*s) => s.count), 15);

        let res = mu.tryLock();
        if (res != null) {
            res.count = res.count + 1;
            res.release();
        }
        expect_eq(mu.withLock((*s) => s.count), 16);
        expect_eq(mu.isLocked(), false);
    });

    test("Bare Mutex", () => {
        let mu = Mutex();
        expect_eq(mu.isLocked(), false);
        mu.lock();
        expect_eq(mu.isLocked(), true);
        mu.unlock();
        expect_eq(mu.isLocked(), false);
        expect_eq(mu.tryLock(), true);
        mu.unlock();
    });

    test("RwLock<T> withRead, withWrite, read, write", () => {
        let rw = RwLock(State { value: 100 });
        rw.withWrite((*s) => {
            s.value = 200;
        });
        let val = rw.withRead((s) => s.value);
        expect_eq(val, 200);

        let rg = rw.read();
        expect_eq(rg.value, 200);
        expect_eq(rw.isReadLocked(), true);
        rg.release();

        let wg = rw.write();
        wg.value = 300;
        expect_eq(rw.isWriteLocked(), true);
        wg.release();

        expect_eq(rw.withRead((s) => s.value), 300);
    });

    test("Bare RwLock", () => {
        let rw = RwLock();
        rw.read();
        expect_eq(rw.isReadLocked(), true);
        rw.unlockShared();
        expect_eq(rw.isReadLocked(), false);

        rw.write();
        expect_eq(rw.isWriteLocked(), true);
        rw.unlock();
        expect_eq(rw.isWriteLocked(), false);
    });

    test("Atomic<i32> arithmetic, bitwise and CAS operations", () => {
        let atom = Atomic(10);
        expect_eq(atom.load(), 10);

        atom.store(20);
        expect_eq(atom.load(), 20);

        let old = atom.swap(30);
        expect_eq(old, 20);
        expect_eq(atom.load(), 30);

        let ok1 = atom.cas(30, 40);
        expect_eq(ok1, true);
        expect_eq(atom.load(), 40);

        let ok2 = atom.compareAndSwap(99, 50);
        expect_eq(ok2, false);
        expect_eq(atom.load(), 40);

        let prev_add = atom.add(5);
        expect_eq(prev_add, 40);
        expect_eq(atom.load(), 45);

        let prev_sub = atom.sub(5);
        expect_eq(prev_sub, 45);
        expect_eq(atom.load(), 40);

        atom.inc();
        expect_eq(atom.load(), 41);

        atom.dec();
        expect_eq(atom.load(), 40);

        atom.and(15);
        expect_eq(atom.load(), 8);

        atom.or(1);
        expect_eq(atom.load(), 9);

        atom.xor(1);
        expect_eq(atom.load(), 8);

        atom.clear();
        expect_eq(atom.load(), 0);
    });

    test("Atomic<bool> testAndSet and clear", () => {
        let flag = Atomic(false);
        expect_eq(flag.load(), false);
        expect_eq(flag.testAndSet(), false);
        expect_eq(flag.load(), true);
        flag.clear();
        expect_eq(flag.load(), false);
    });

    test("Shared(Mutex(T)) Interior Mutability", () => {
        let shared_mu = Shared(Mutex(Counter { count: 100 }));
        shared_mu.withLock((*c) => {
            c.count = c.count + 50;
        });
        let res = shared_mu.withLock((*c) => c.count);
        expect_eq(res, 150);
    });
}
