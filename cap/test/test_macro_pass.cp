// cap/test/test_macro_pass.cp
// Comprehensive Phase 4 Test for Builtin Macros (@Clone, @Comparable, @Hash, @ToString, @Json, @Table)
// Aggregator @derive(...), Batch Syntax @[...], and Third-Party Template Macros

import { expect_eq } from "cap:test";
import { print } from "cap:io";

// 1. User-Defined Custom Template Macro
macro Accessor(target, field) {
    fn get_${field}(_): string {
        return _.${field};
    }
}

// 2. User-Defined Legacy Empty Tag Macros
macro Tag(target) {}
macro Environment(target) {}

// 3. Class using @Json, @Table("t_users"), and custom @Tag("UserModule")
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

// 4. Two-Tier TypeInfo: Mutual/Nested Symbolic Reference without deadlock
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

// 5. Class using @ToString and custom @Environment without args
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

// 6. User-Defined Template Macro on Book
@Accessor("title")
class Book {
    title: string;
    author: string;

    fn init(title: string, author: string) {
        _.title = title;
        _.author = author;
    }
}

// 7. Full Batch Annotation testing Clone, Comparable, Hash, ToString, Json
@[Clone, Comparable, Hash, ToString, Json]
class OrderItem {
    sku: string;
    price: f64;
    quantity: i32;
}

// 8. Batch Annotation syntax @[Clone, ToString, Json]
@[Clone, ToString, Json]
class Point {
    x: i32;
    y: i32;

    fn init(x: i32, y: i32) {
        _.x = x;
        _.y = y;
    }
}

// 9. Class using @[Debug, Clone]
@[Debug, Clone]
class Customer {
    name: string;
    level: i32;
}

// 10. Custom Iterator Stream
class NumberStream {
    current: i32;
    limit: i32;

    fn init(start: i32, limit: i32) {
        _.current = start;
        _.limit = limit;
    }

    fn next(*_): ?i32 {
        if (_.current >= _.limit) {
            return null;
        }
        let val = _.current;
        _.current = _.current + 1;
        return val;
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

    @print("=== [Test 6] Third-Party Template Macro @Accessor ===");
    let book = new Book("Design Patterns", "GoF");
    let book_title = book.get_title();
    @print("Book get_title: {s}", book_title);
    expect_eq(book_title, "Design Patterns");

    @print("=== [Test 7] Full Batch Annotation @[Clone, Comparable, Hash, ToString, Json] ===");
    let item1 = new OrderItem("SKU-APPLE", 9.99, 10);
    let item2 = new OrderItem("SKU-APPLE", 9.99, 10);
    let item3 = new OrderItem("SKU-BANANA", 5.50, 20);

    // 1. ToString
    let item1_str = item1.toString();
    @print("OrderItem 1 toString: {s}", item1_str);
    expect_eq(item1_str, "OrderItem(sku=SKU-APPLE, price=9.99, quantity=10)");

    // 2. Json
    let item1_json = item1.toJson();
    @print("OrderItem 1 toJson: {s}", item1_json);
    expect_eq(item1_json, "{\"sku\":\"SKU-APPLE\",\"price\":9.99,\"quantity\":10}");

    // 3. Clone
    let item1_cloned = item1.clone();
    @print("OrderItem 1 cloned: {s}", item1_cloned.toString());
    expect_eq(item1.eq(item1_cloned), true);

    // 4. Comparable (eq & cmp)
    expect_eq(item1.eq(item2), true);
    expect_eq(item1.eq(item3), false);
    expect_eq(item1.cmp(item2), 0);
    expect_eq(item1.cmp(item3), -1);
    expect_eq(item3.cmp(item1), 1);

    // 5. Hash
    let h1 = item1.hash();
    let h2 = item2.hash();
    let h3 = item3.hash();
    @print("Hash item1: {d}, Hash item2: {d}, Hash item3: {d}", h1, h2, h3);
    expect_eq(h1 == h2, true);
    expect_eq(h1 == h3, false);

    @print("=== [Test 8] Batch Annotation @[Clone, ToString, Json] ===");
    let pt1 = new Point(100, 200);
    let pt2 = pt1.clone();
    expect_eq(pt2.toString(), "Point(x=100, y=200)");
    expect_eq(pt2.toJson(), "{\"x\":100,\"y\":200}");

    @print("=== [Test 9] @[Debug] Derivation (fmt_debug & toString) ===");
    let cust = new Customer("Bob", 5);
    @print("Customer toString: {s}", cust.toString());
    expect_eq(cust.toString(), "Customer(name=Bob, level=5)");
    @print("Customer fmt_debug: {s}", cust.fmt_debug());
    expect_eq(cust.fmt_debug(), "Customer(name=Bob, level=5)");

    @print("=== [Test 10] Custom Iterator Stream for...of Loop ===");
    let mut stream = new NumberStream(1, 6);
    let mut stream_sum = 0;
    for n of stream {
        stream_sum = stream_sum + n;
    }
    @print("Iterator Stream Sum: {d}", stream_sum);
    expect_eq(stream_sum, 15);

    @print("=== All Macro Metaprogramming tests passed 100%! ===");
}
