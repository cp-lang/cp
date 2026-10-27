import { print } from "cap:io";
import { spawn, join } from "cap:parallel";

@parallel(4)
async fn main() {
  let shared_prefix = "Prefix:";
  let secret_token = "OwnedSecret-42";

  // 1. 多核 Worker 线程池并发执行异步任务（支持只读捕获）
  let task1 = spawn(async () => {
    print(`${shared_prefix} Running Task 1`);
    return "Hello from Task 1";
  });

  // 2. 使用 move 将变量所有权安全转移至并发任务中
  let task2 = spawn(async () => {
    let owned = move secret_token;
    print(`Task 2 received ${owned}`);
    return "Hello from Task 2";
  });

  // 3. 使用 spawn.cpu 派发重度计算任务至专用 OS 计算线程
  let cpu_task = spawn.cpu(() => {
    let mut sum = 0;
    let mut i = 0;
    while (i < 1000) {
      sum = sum + i;
      i = i + 1;
    }
    return "CPU Compute Done";
  });

  // 等待多任务并行汇聚结果
  let [msg1, msg2, msg3] = await join(task1, task2, cpu_task);

  print(msg1);
  print(msg2);
  print(msg3);
  print("Hello, Concurrent Plus!");
}
