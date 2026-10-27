import { print } from "cap:io";
import { spawn, join } from "cap:parallel";

async fn main() {
  // 并发执行两个任务
  let task1 = spawn(async () => {
    return "Hello from Task 1";
  });

  let task2 = spawn(async () => {
    return "Hello from Task 2";
  });

  // 等待结果
  let [msg1, msg2] = await join(task1, task2);

  print(msg1);
  print(msg2);
  print("Hello, Concurrent Plus!");
}
