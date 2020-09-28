# `geniter`: iterators out of generators.

This crate was an experiment to work with generators as iterators within for loops.

## Background

Generators in Rust are a kind of stackless coroutine, which are written like closures and can function like iterators. For example:

```rust
let generator = || {
    for i in 0..10i32 {
        yield i;
    }
}
```

This generator returns the values `0` through `9` before returning.

## For Loops

It would be great if you could drive an iterator in a for loop. 

```rust 
for value in generator {
    println!("{}", value);
}
```

But this will fail to compile, as generators are _not_ iterators and don't implement `IntoIterator`. `geniter` provides a macro to help

```rust
for value in geniter!(generator) {
    println!("{}", value);
}
```

## Non-void resume arguments

Generators don't implement `IntoIterator` for good reason: generators can be _resumed_ with arguments, unlike iterators. 

Take this example:

```rust
let mut read_line = |mut arg| {
    let mut buf = String::new();
    loop {
        match arg {
            '\0' => { 
                // break on null characters and end sequence
                arg = yield Poll::Ready(buf.clone());
                return;
            }, 
            '\n' => {
                // break on new lines and clear buffer
                arg = yield Poll::Ready(buf.clone());
                buf.clear();
            }
            _ => { 
                // buffer character and yield pending
                buf.push(arg);
                arg = yield Poll::Pending;
            }
        }
    }
}
```

This is an asynchronous version of `read_line` as a generator. This is how we drive it today:

```rust
let text = "first line\nsecond line\nlast line\0";
for ch in text.chars() {
    match Generator::resume(Pin::new(&mut read_line), ch) {
        GeneratorState::Yielded(yielded) => {
            match yielded {
                Poll::Ready(line) => println!("{}", line);
                Poll::Pending => (), // nothing else to do
            }
        },
        GeneratorState::Returned(_) => break,
    }
}
```

This example highlights how `geniter` allows generators in for loops even if they don't take `()` as their resume argument: the macro requires you to provide an iterator to _bind_ to the resume arguments of the generator.

```rust
for line in geniter!(text.chars() => read_line) {
    println!("{}", line.await); //note: this isn't valid, read_line doesn't return a future, just a Poll result. It could though!
}
```

But there's another good reason that generators aren't `IntoIterator`: they can return anything, iterators only every return `Some(Item)` or `None`. `geniter` supports this using a callback called `then`, which is executed when the generator is returned.

```rust
let then = |returned| println!("returned: {}", returned);
for line in geniter!(text.chars() => read_line, then) {
    println!("{}", line.await);
}
```

## Random thoughts

- I'm convinced that we can use the result of a `for` loop expression more gracefully, eg allowing `break` to return a value that can be assigned to a variable binding or passed into a function. For example, consider an iterator bound to generator that returns an iterator of futures to evaluate the loop concurrently or to schedule their evaluation. 
- Generators have to be pinned. This seems to put limits on how useful generators can be as general purpose iterators or streams, but I don't know all of the implications.
- It would be great to pipe generators from one to the other, eg `for data in geniter!(tcp_stream => decode => parse)`. 
- This is tightly coupled to [async iteration semantics](https://blog.yoshuawuyts.com/async-iteration/#how-would-tasks-be-spawned) and [for await loops](https://without.boats/blog/for-await-i/). 

## Further Reading

- [Unified Coroutines RFC](https://github.com/rust-lang/rfcs/pull/2781)
- [Generator integration with for loops](https://internals.rust-lang.org/t/pre-rfc-generator-integration-with-for-loops/6625)
- [Propane: an experimental generator syntax for Rust](https://without.boats/blog/propane/)
