# FastCmpStr in Rust
The Rust standard `String` is excellent, but structurally its just a `Vec<u8>` with some string related functions. This means all data is on the heap. A single deference is usually never a problem but given a `Vec<String>` and we want to, as an example do a `contains` operation... Suddenly you need to dereference every single `String` which is not **cache efficient**. And as usual, today we are rarely computationally bound but more often than not memory latency bound...

So! By compromising on max length to `u32` instead of `usize` we have 12 bytes to spare compared to a `Vec<u8>` 8+8+8 (length + capacity + ptr) default size. We can take 2 to use for an _offset capacity_ field, allowing for more efficient mutability (similar to `Vec`) and the rest 10 for storing the first 10 characters (actually bytes due to UTF-8) of the String.

Notice the layout of our implementation:
```
Str (size 24, alignment 8)
| Offset | Name                  | Size |
| ------ | --------------------- | ---- |
| 0      | len: u32              | 4    |
| 4      | capacity_offset: u16  | 2    |
| 6      | prefix: [u8; 10]      | 10   |
| 16     | suffix: *mut u8       | 8    |
```
The size and alignment is the same as String, but we have split the data into a static prefix and a dynamic suffix. The length of the allocated suffix pointer will always be `max(0, len + capacity_offset - 10)`.

## Why is this faster at comparision?
As mentioned above, due to **cache efficiencies**. Lets compare the following:

Using `std::string::String`:
```rust
    let a = String::from("Hi! This is a message from the future!");
    let b = String::from("Hi! This IS a message");
    //                             ^

    if a.starts_with(b) {
        // Do stuff...
    }
```

Using our implementation:
```rust
    let a = Str::from("Hi! This is a message from the future!");
    let b = Str::from("Hi! This IS a message");
    //                          ^

    if a.starts_with(b) {
        // Do stuff...
    }
```

Here we need to walk char by char to check the starts_with operation. For `std::string::String` does this mean *TWO* indirections into _potentially_ two very different places in memory. For our implementation and this example where is a difference in the 10th byte, comparing in the stack is enough and we never have to do an indirection

This ofcourse can be even more notable when you have a `Vec<String>` vs `Vec<Str>` and want to find all elements that fullfills any comparision. Using String requires _potentially_ indirection at every step. Our implementation _potentially_ does not.

### Bonus: Small string optimization
Infact, this is also a small string optimization, which notably `std::string::String` does not do... Although an inefficient one since only 10 out of our 24 bytes are usuable. Any strings with byte length smaller or equal to 10 will be kept entirely on the stack, which means no heap allocation is needed.

## Benchmarks against `std::string::String`
Run `cargo bench` for details and HW specific numbers. The numbers should not be seen as expectations due to potentially wildly different circumstances and HW, only as examples.

### Equality on same length string

In general: We see a **~2.5x** improvement in performance for random strings in an equality test. 
If there all strings have atleast 10 bytes in common and we need to dereference our suffix we see a **~0.8x** decrease in performance.


### Vec.contains with 10k samples

A **~3x** improvement on random strings with randomness in the first 10 chars over `std::string::String`.
A **0.95x** degregation when the string is static in the first 32/64 chars.

### starts_with

TODO

## TLDR: When should I use this?

Whenever you compare strings alot that dont tend to all start with the same 10 bytes.

Ofcourse this is just a "learning" project. I bet there are multiple better crates available... So in practice never?