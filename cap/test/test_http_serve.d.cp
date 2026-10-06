// Type definitions for CP

import { test, expect_eq } from "cap:test";
import { serve, Response, Request, json } from "cap:http";
import { print } from "cap:io";
export fn handle(req: Request): Response;
