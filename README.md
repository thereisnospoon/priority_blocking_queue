# Priority blocking queue

Basic implementation of thread-safe priority queue that blocks on `pop` from
empty queue. The implementation wraps `std::collections::BinaryHeap`.