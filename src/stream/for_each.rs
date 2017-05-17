use {Async, Future, IntoFuture, Poll};
use stream::Stream;

extern crate rand;

/// A stream combinator which executes a unit closure over each item on a
/// stream.
///
/// This structure is returned by the `Stream::for_each` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct ForEach<S, F, U> where U: IntoFuture {
    stream: S,
    f: F,
    fut: Option<U::Future>,
    name: &'static str,
}

pub fn new<S, F, U>(s: S, f: F) -> ForEach<S, F, U>
    where S: Stream,
          F: FnMut(S::Item) -> U,
          U: IntoFuture<Item = (), Error = S::Error>,
{
    ForEach {
        stream: s,
        f: f,
        fut: None,
        name: "NONE",
    }
}


/// BLAH
pub fn with_name<S, F, U>(s: S, f: F, name: &'static str) -> ForEach<S, F, U>
    where S: Stream,
          F: FnMut(S::Item) -> U,
          U: IntoFuture<Item = (), Error = S::Error>,
{
    ForEach {
        stream: s,
        f: f,
        fut: None,
        name,
    }
}

impl<S, F, U> Future for ForEach<S, F, U>
    where S: Stream,
          F: FnMut(S::Item) -> U,
          U: IntoFuture<Item= (), Error = S::Error>,
{
    type Item = ();
    type Error = S::Error;

    fn poll(&mut self) -> Poll<(), S::Error> {
        println!("for_each {:?}: Entering for_each poll!", self.name);
        loop {
            if let Some(mut fut) = self.fut.take() {
                if try!(fut.poll()).is_not_ready() {
                    self.fut = Some(fut);
                    println!("for_each {:?}: The future stored in for_each is not ready!", self.name);
                    return Ok(Async::NotReady);
                }
            }
            use stream::for_each::rand::Rng;
            let stackframe: u32 = rand::thread_rng().gen();
            println!("for_each {:?}: Polling the stream for the next future! {:?}", self.name, stackframe);
            let r = self.stream.poll();
            println!("for_each {:?}: Polled! {:?}", self.name, stackframe);

            let r = match r {
                Ok(Async::Ready(r)) => r,
                Ok(Async::NotReady) => { println!("for_each {:?}: Not ready, so returning. {:?}", self.name, stackframe); return Ok(Async::NotReady) },
                Err(e) => return Err(e),
            };

            match r {
                Some(e) => self.fut = Some((self.f)(e).into_future()),
                None => {
                    println!("for_each {:?}: Return ready", self.name);
                    return Ok(Async::Ready(()))
                    },
            }
            println!("for_each {:?}: Got future, looping.", self.name);
        }
    }
}
