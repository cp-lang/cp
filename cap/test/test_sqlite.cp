import { Database } from "cap:sqlite";

fn test_sqlite_sync(): ?void {
    @print("Testing cap:sqlite initialization...");
    // 1. Initialize DB with memory (auto-drop will trigger when this function ends)
    const db = new Database(":memory:");
    
    // 2. Execute SQL
    db.exec("CREATE TABLE users (id INT, name TEXT)")?;
    db.exec("INSERT INTO users VALUES (1, 'CP-BEAM Master')")?;
    
    // 3. Query and verify
    const name = db.query_string("SELECT name FROM users WHERE id = 1")?;
    
    if (name == "CP-BEAM Master") {
        @print("Data verification PASSED!");
    } else {
        @print("Data verification FAILED!");
    }

    @print("SQLite executed successfully. Exiting function to trigger Auto-Drop...");
}

export fn main(): ?void {
    test_sqlite_sync()?;
}
