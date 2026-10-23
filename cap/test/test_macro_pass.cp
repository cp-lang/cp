// cap/test/test_macro_pass.cp
// Comprehensive Phase 4 Test for Type-Directed Macros & Anti-Cycle System

import { expect_eq } from "cap:test";
import { print } from "cap:io";

// 1. User-Defined Custom Macros
macro Tag(target) {}
macro Environment(target) {}

// 2. Class using @Json, @Table("t_users"), and custom @Tag("UserModule")
@Json
@Table("t_users")
@Tag("UserModule")
class User {
    id: i32;
    name: string;

    fn init(id: i32, name: string) {
        _.id = id;
        _.name = name;
    }
}

// 3. Two-Tier TypeInfo: Mutual/Nested Symbolic Reference without deadlock
@Json
@Table
class Order {
    orderId: i32;
    user: User;

    fn init(orderId: i32, move user: User) {
        _.orderId = orderId;
        _.user = user;
    }
}

// 4. Class using @ToString and custom @Environment without args
@ToString
@Environment
class ServerConfig {
    host: string;
    port: i32;

    fn init(host: string, port: i32) {
        _.host = host;
        _.port = port;
    }
}

fn main(): void {
    @print("=== [Test 1] @Json serialization method derivation ===");
    let u = new User(1, "Alice");
    let user_json = u.toJson();
    @print("User JSON: {s}", user_json);
    expect_eq(user_json, "{\"id\":1,\"name\":\"Alice\"}");

    @print("=== [Test 2] @Table ORM method derivation ===");
    let table_name = u.getTableName();
    @print("User Table: {s}", table_name);
    expect_eq(table_name, "t_users");

    @print("=== [Test 3] Custom macro @Tag method derivation ===");
    let tag_name = u.getTag();
    @print("User Tag: {s}", tag_name);
    expect_eq(tag_name, "UserModule");

    @print("=== [Test 4] Two-Tier TypeInfo Nested @Json Introspection ===");
    let u2 = new User(2, "Bob");
    let o = new Order(1001, u2);
    let order_json = o.toJson();
    @print("Order JSON: {s}", order_json);
    expect_eq(order_json, "{\"orderId\":1001,\"user\":{\"id\":2,\"name\":\"Bob\"}}");

    let order_table = o.getTableName();
    @print("Order Table: {s}", order_table);
    expect_eq(order_table, "order");

    @print("=== [Test 5] @ToString & Default Custom Macro Derivation ===");
    let cfg = new ServerConfig("localhost", 8080);
    let cfg_str = cfg.toString();
    @print("ServerConfig toString: {s}", cfg_str);
    expect_eq(cfg_str, "ServerConfig(host=localhost, port=8080)");

    let env_name = cfg.getEnvironment();
    @print("ServerConfig Environment: {s}", env_name);
    expect_eq(env_name, "Environment");

    @print("=== All Phase 4 Type-Directed Macro tests passed! ===");
}
