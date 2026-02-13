# Backpressure

## OverflowPolicy

```rust
enum OverflowPolicy {
    Fail,
    Drop,
    Wait
}
```

- `Fail` if message cannot be delivered the receivers die, as undelivered
   messages are not acceptable
- `Drop` - message channel is full. If there are multiple channels (multiple)
   actors subscribed to the same topic, there will be attempt to deliver to
   each actor siscribed for the same topic. Drop will happen only for
   acros that are full.


### Actor behavior (Actor->Broker)

Check in `Context` on `send`.

### Broker behavior (Broker->Actor)


```rust
Topic
```
