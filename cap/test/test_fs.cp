import { writeFileSync, readFileSync, exists, mkdir, remove, rmdir, rename, copyFile, statSize, isDirectory } from "cap:fs";
import { expect_eq } from "cap:test";
import { print } from "cap:io";

async fn main(): void {
    let test_dir = "test_output_dir";
    let test_file = "test_output_dir/hello.txt";
    let copied_file = "test_output_dir/hello_copy.txt";
    let renamed_file = "test_output_dir/hello_renamed.txt";
    let content = "Hello from CP standard library!";

    if (exists(test_dir)) {
        if (exists(test_file)) remove(test_file);
        if (exists(copied_file)) remove(copied_file);
        if (exists(renamed_file)) remove(renamed_file);
        rmdir(test_dir);
    }

    mkdir(test_dir);
    expect_eq(exists(test_dir), true);
    expect_eq(isDirectory(test_dir), true);

    writeFileSync(test_file, content);
    expect_eq(exists(test_file), true);
    expect_eq(isDirectory(test_file), false);
    
    // Check size. "Hello from CP standard library!" is 31 bytes.
    let size: i32 = statSize(test_file);
    expect_eq(size, 31);

    let read_content = readFileSync(test_file);

    // copy
    copyFile(test_file, copied_file);
    expect_eq(exists(copied_file), true);
    expect_eq(statSize(copied_file), 31);
    
    // rename
    rename(copied_file, renamed_file);
    expect_eq(exists(copied_file), false);
    expect_eq(exists(renamed_file), true);
    
    // remove files
    remove(test_file);
    expect_eq(exists(test_file), false);
    remove(renamed_file);
    expect_eq(exists(renamed_file), false);
    
    // remove dir
    rmdir(test_dir);
    expect_eq(exists(test_dir), false);

    @print("🎉 All cap:fs sync methods verified successfully!");
}
