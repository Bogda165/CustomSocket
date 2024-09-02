use std::marker::PhantomData;

pub struct MyZip<A, B, F, T> {
    a: A,
    b: B,
    func: F,
    _member: PhantomData<T>,
}

impl<A, B, F, T> Iterator for MyZip<A, B, F, T>
where
    A: Iterator,
    B: Iterator,
    F: Fn(A::Item, B::Item) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.a.next(), self.b.next()) {
            (Some(a_val), Some(b_val)) => {
                Some((self.func)(a_val, b_val))
            }
            _ => None
        }
    }
}

pub trait MyZipIterator: Iterator {
    fn my_zip<B, F, T>(self, other: B, function: F) -> MyZip<Self, B, F, T>
    where
        Self: Sized,
        B: Iterator,
        F: Fn(Self::Item, B::Item) -> T,
    {
        MyZip {
            a: self,
            b: other,
            func: function,
            _member: PhantomData::<T>,
        }
    }
}

impl<I: Iterator> MyZipIterator for I {}


//MACROS

macro_rules! println_t {
    ($($arg:tt)*) => {
        println!("{}: {}", thread::current().name().unwrap_or("unknown"), format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use std::ops::Add;
    use super::*;

    #[test]
    fn it_works() {
        let vec1 = vec![1, 2, 3, 4];
        let vec2 =  vec![5, 6, 7, 8];
        let clos = |mut a: &i32, b: &i32| { a + b };

        let answer: Vec<_> = vec1
                                    .iter()
                                    .zip(vec2.iter()).map(|(a, b)|clos(a, b))
                                    .collect();

        assert_eq!(vec![6, 8, 10, 12], answer);
    }
}
