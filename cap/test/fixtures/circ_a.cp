import { NodeB } from "./circ_b.cp";

export class NodeA {
    id: i32;
    b: ?*NodeB;

    fn get_id(): i32 {
        return this.id;
    }

    fn attach_b(*this, target: *NodeB): void {
        this.b = target;
    }
}
