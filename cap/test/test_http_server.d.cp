// Type definitions for CP

import { test, expect_eq } from "cap:test";
import { listen, Server } from "cap:net";
import { print } from "cap:io";
export async fn acceptLoop(s: Server): void;
export async fn startServer(): void;
