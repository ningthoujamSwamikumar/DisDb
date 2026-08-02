# Design Doc v1
Create in memory key value store by using a hash map.
- start a tcp server ✅
- send message from a tcp client cli ✅
- frame the byte stream by using a delimiter ✅
- receive the message, process it and return response ✅
- write unit and integration test for the existing functionality ✅
- stress test and break the current system ✅
```txt
Executed 200000 operations in 13.533667194s
Througput: 14778 ops/sec
let num_client = 1000;
let ops_per_client = 100; // 1 read and 1 write for each of ops

Executed 2000000 operations in 148.384691483s
Througput: 13478 ops/sec
Test run through 1000 clients, 2000 ops/client

Executed 2000000 operations in 138.632710194s
Througput: 14427 ops/sec
Test run through 10000 clients, 200 ops/client

Executed 20000000 operations in 1608.221710213s
Througput: 12436 ops/sec
Test run through 10000 clients, 2000 ops/client

Executed 20000000 operations in 1598.653715494s
Througput: 12511 ops/sec
Test run through 10000 clients, 2000 ops/client
```

# DD v2
Add concurrency - multiple clients to connect and get/set concurrently.
Benchmark/profile the limit.
- add concurrency to the server ✅
- use lock free data sharing
- length delimited framing from scratch
- length delimited framing using tokio-utils
- protobuf serialization instead of json string to improve serialization and deserialization

# DD v3
What happens when crashed? The data shouldn't be lost.
Benchmark the new version.

# DD V4
What to build next?
