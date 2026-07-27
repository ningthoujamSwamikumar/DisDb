# Design Doc v1
Create in memory key value store by using a hash map.
- start a tcp server ✅
- send message from a tcp client cli ✅
- frame the byte stream by using a delimiter ✅
- receive the message, process it and return response ✅
- write unit test for the existing functionality
- stress test and break the current system

# DD v2
Add concurrency - multiple clients to connect and get/set concurrently.
Benchmark/profile the limit.

# DD v3
What happens when crashed? The data shouldn't be lost.
Benchmark the new version.

# DD V4
What to build next?
