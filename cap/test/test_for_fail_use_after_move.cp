// Attempting to use a collection after 'for move x of list' should fail with UseOfMovedVar at compile-time
async fn main(): void {
    let list: i32[] = [1, 2, 3];
    for move x of list {
        let val = x;
    }
    let next = list;
}
