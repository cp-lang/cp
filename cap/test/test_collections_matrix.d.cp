// Type definitions for CP

import { test, expect_eq } from "cap:test";
import { print } from "cap:io";
import { VecDeque, BinaryHeap, BTreeMap, BTreeSet, HashMap, HashSet } from "cap:collections";
export fn mutate_deque(q: &mut VecDeque, val: i32): void;
export fn query_deque(q: &VecDeque): usize;
export fn mutate_heap(h: &mut BinaryHeap, val: i32): void;
export fn mutate_btree(b: &mut BTreeMap, k: string, v: i32): void;
