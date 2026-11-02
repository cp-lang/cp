import { print } from "cap:io";

fn test_loop_control() {
    let mut i: usize = 0;
    let mut sum: usize = 0;
    while (i < 10) {
        i = i + 1;
        if (i == 3) {
            continue;
        }
        if (i == 7) {
            break;
        }
        sum = sum + i;
    }
    // i went through: 1 (sum=1), 2 (sum=3), 3 (continue), 4 (sum=7), 5 (sum=12), 6 (sum=18), 7 (break)
    @print("Loop control sum: {d}", sum);
}

fn test_array_repeat() {
    let zero_buf: [u8; 8] = [0; 8];
    let ff_buf: [u8; 4] = [0xFF; 4];
    @print("Array repeat zero: {d}, ff: {d}", zero_buf[0], ff_buf[0]);
}

fn main() {
    test_loop_control();
    test_array_repeat();
    @print("Bun features verified successfully!");
}
