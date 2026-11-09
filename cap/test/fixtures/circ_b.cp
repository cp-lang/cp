import { NodeA } from "./circ_a.cp";

export class NodeB {
    tag: i32;
    a: ?*NodeA;

    fn get_tag(): i32 {
        return this.tag;
    }

    fn attach_a(*this, target: *NodeA): void {
        this.a = target;
    }
}
