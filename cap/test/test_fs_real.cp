import { print } from "cap:io";
import { readFileSync, writeFileSync } from "cap:fs";

fn main(): void {
    @print("--- FS Standard Library Test ---");
    const path = "test_data.txt";
    const data = "CP-BEAM File System Rocks!";
    
    @print("Writing to {s}...", path);
    writeFileSync(path, data);
    
    @print("Reading from {s}...", path);
    const read_data = readFileSync(path);
    @print("Read: {s}", read_data);
    
    if (read_data == data) {
        @print("FS SUCCESS!");
    }
}
