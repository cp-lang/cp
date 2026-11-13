// lib/collections/index.cp
// Unified Public Exports for cap:collections
// Architecture:
//   - Built-in high-level containers (provided by cpc runtime):
//       VecDeque, BinaryHeap, BTreeMap, BTreeSet, HashMap, HashSet, Map, Set, Dict
//   - bounded_array.cp : Zero-heap stack-allocated BoundedArray<T, CAP>
//   - baby_list.cp     : Compact u32-indexed BabyList<T> and OffsetByteList
//   - linear_fifo.cp   : Power-of-two ring-buffer LinearFifo<T>
//   - hive_array.cp    : Hardware @ctz/@popCount BitSet64 and O(1) HiveArray<T, CAP> slot pool
//   - string_map.cp    : Length-filtered FastStringMap<V, CAP> (exact & ASCII case-insensitive)

export { BoundedArray } from "./bounded_array";
export { BabyList, OffsetByteList } from "./baby_list";
export { LinearFifo } from "./linear_fifo";
export { BitSet64, HiveArray } from "./hive_array";
export { StringEntry, FastStringMap } from "./string_map";
