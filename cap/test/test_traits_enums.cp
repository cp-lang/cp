trait Worker {
    fn work(this, batch: usize): void;
}

enum Task {
    Compute { batch: usize },
    Idle
}

class AIWorker {
    id: i32
}

impl Worker for AIWorker {
    fn work(this, batch: usize): void {
        @print("AIWorker {d} computing batch {d}", this.id, batch);
    }
}

fn main(): ?void {
    let task: Task = Task.Compute { batch: 100 };
    let worker = new AIWorker();
    worker.id = 1;
    match (task) {
        .Compute |fields| => {
            worker.work(fields.batch);
        }
        .Idle => {
            @print("Worker idle.");
        }
    }
}