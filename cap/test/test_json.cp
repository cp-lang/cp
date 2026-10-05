import { json } from "cap:json";
import { print } from "cap:io";

class Config {
    port: i32,
    host: string,
}

fn main(): void {
    @print("--- JSON Module Test ---");
    
    // Test Parse
    let json_str = "";
    __zig__ { ctx.json_str = "{\"port\": 9000, \"host\": \"127.0.0.1\"}"; }
    const conf: Config = @json_parse(json_str, Config);
    
    @print("Parsed Port: {}", conf.port);
    @print("Parsed Host: {s}", conf.host);
    
    // Test Stringify
    conf.port = 9001;
    const str = @json_stringify(conf);
    @print("Stringified: {s}", str);
    
    if (conf.port == 9001) {
        @print("JSON SUCCESS!");
    }
}
