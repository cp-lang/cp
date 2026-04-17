import { writeFileSync, readFileSync, exists, mkdir, remove } from "cap:fs";
import { print } from "cap:io";

fn main(): ?void {
    @print("--- File System Test ---");

    let test_dir = "./test_output_dir";
    let test_file = "./test_output_dir/hello.txt";
    let content = "Hello from CP standard library!";

    @print("Creating directory: {s}", test_dir);
    mkdir(test_dir);

    @print("Writing file: {s}", test_file);
    writeFileSync(test_file, content);

    if (exists(test_file)) {
        @print("File exists confirmed.");
    } else {
        @print("Error: File does not exist!");
    }

    @print("Reading file back...");
    let read_content = readFileSync(test_file);
    @print("Content: {s}", read_content);

    @print("Cleaning up...");
    // remove(test_dir); 
    @print("Test complete.");
}
