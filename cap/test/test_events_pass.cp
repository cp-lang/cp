import { print } from "cap:io";
import { EventEmitter, Event } from "cap:events";

async fn main() {
    let mut ee = new EventEmitter();

    let mut count = 0;
    let unsub = ee.on("data", (val, extra) => {
        print("Received event data");
    });

    ee.emit("data");

    // Test Event signal
    let mut evt = new Event();
    print(evt.isSet()); // false
    evt.set();
    print(evt.isSet()); // true
    evt.reset();
    print(evt.isSet()); // false

    print("EventEmitter and Event verified!");
}
