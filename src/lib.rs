#![allow(dead_code, unused_imports)]
#![feature(generators, generator_trait)]
use std::ops::{Generator, GeneratorState, Shl, Shr};
use std::pin::Pin;
use std::marker::Unpin;
use std::iter; 

pub fn void () -> impl Iterator<Item = ()> {
    std::iter::from_fn(|| Some(()))
}

pub struct GenIter<G, I, F> {
    gen:  G, 
    iter: I,
    then: F, 
}

impl<G, I, F, R> Iterator for GenIter<G, I, F> 
where G: Generator<R> + Unpin, 
      I:Iterator <Item = R>,
      F:Fn(<G as Generator<R>>::Return) {
    type Item = <G as Generator<R>>::Yield;
    fn next(&mut self) -> Option<Self::Item> {
        let next_resume = self.iter.next()?;
        match Generator::resume(Pin::new(&mut self.gen), next_resume) {
            GeneratorState::Yielded(yielded) => Some(yielded), 
            GeneratorState::Complete(complete) => {
                (self.then)(complete);
                None
            }
        }
    }
}

pub fn bind<G, I, F> (iter:I, generator:G, then:F) -> GenIter<G, I, F> {
    GenIter { gen: generator, iter, then }
}

#[macro_export]
macro_rules! geniter {
    ($it:expr =>  $gen:expr) => {
        $crate::bind($it, $gen, |_| {})
    };
    ($it:expr => $gen:expr, $then:expr) => {
        $crate::bind(std::iter::IntoIterator::into_iter($it), $gen, $then)
    }
}

#[cfg(test)]
mod tests {
    use crate::{void, geniter};
    fn sanity_check() {
        let generator = || {
            for i in 0..10i32 {
                yield i;
            }
        };

        for value in geniter!(void() => generator) {
            println!("{}", value);
        }

        let generator = |mut arg:&str| {
            loop {
                if arg == "exit" {
                    break;
                } else {
                    println!("received {}", arg);
                }
                arg = yield;
            }
            return "returned string"; 
        };

        let then = |arg|{ println!("handled {}", arg)};
        for _ in geniter!(vec!["first", "second", "exit"] => generator, then) {
            println!("...");
        }
    }
}
