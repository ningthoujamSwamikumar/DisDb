# Design Doc v1
Create in memory key value store by using a hash map.
- start a tcp server
- send message from a tcp client cli
- receive the message, deserialize it, and process the command


# DD v2
Add concurrency - multiple clients to connect and get/set concurrently.
Benchmark/profile the limit.

# DD v3
What happens when crashed? The data shouldn't be lost.
Benchmark the new version.

# DD V4
What to build next?
