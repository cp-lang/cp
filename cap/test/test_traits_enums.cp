trait Worker {
    fn work(this: any, batch: usize): void;
}

enum Task {
    Compute { batch: usize },
    Idle
}

class AIWorker {
    id: i32
}

impl Worker for AIWorker {
    fn work(this: any, batch: usize): void {
        @print("AIWorker {d} computing batch {d}", this.id, batch);
    }
}

fn main(): ?void {
    let task: Task = Task.Compute { batch: 100 };
    let worker = new AIWorker();
    worker.id = 1;
    switch (task) {
        .Compute { batch } => {
            worker.work(batch);
        }
        .Idle => {
            @print("Worker idle.");
        }
    }
}