#![feature(generators)]
use geniter::geniter;
use std::task::Poll;

fn main () {
    let text = "first line\nsecond line\nthird line\0\0";
    let read_line = |mut arg| {
        let mut buf = String::new(); 
        loop {
            match arg {
                '\0' => {
                    let _ = yield Poll::Ready(buf.clone()); 
                    return;
                }, 
                '\n' => {
                    arg = yield Poll::Ready(buf.clone()); 
                    buf.clear();
                }, 
                _ => {
                    buf.push(arg); 
                    arg = yield Poll::Pending;
                }
            }
        }
    };
    for line in geniter!(text.chars() => read_line) {
        match line {
            Poll::Ready(line) => println!("{}", line), 
            Poll::Pending => (),
        }
    }
}