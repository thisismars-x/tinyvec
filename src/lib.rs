#![allow(
    unused_variables,
    dead_code,
    unused_imports,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]

use std::default;
use std::fmt::{self, Display, Formatter, Result};
use std::iter;
use std::mem::MaybeUninit;

///Size of heap allocated at once
pub static general_heap: usize = 1024;

#[derive(Debug, PartialEq)]
enum step_iter {
    Not,
    Yes(usize),
}

/// Store a small number of elements on the stack.
///
/// Vec<'_> are inefficient if used with less caution.
/// If some_vec.len() > some_vec.capacity(),
/// some_vec finds a new location on the heap
/// with CAPACITY > some_vec.capacity() * 2
/// and copies all its content into a new location.
///
/// That is why you should always
/// [reserve](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.reserve) space
/// for your vector before any initialization.
///
/// # Usage
///
/// Note that tinyvec type is compatible with only `'Copy-Types'` as now.
/// This is by design. Mostly you would want to use such types. Extend
/// the trait bounds if you feel the need.
///
/// ```rust
/// // Initialize a tinyvec with type i32 and number of elements on stack 64
/// let mut tinyvecwtor: tinyvector<i32, 64> = tinyvec::new();    
///
/// // Zero heap allocations till now
/// for i in 0..=64 {
///     tinyvecwtor.push(i);
/// }
///
/// // len == capacity of stack
/// assert_eq!(tinyvecwtor.len(), tinyvecwtor.capacity().0);
///
/// // adding more values, pushes them to heap now
/// tinyvecwtor.push(100);
///
/// // To print your tinyvec
/// println!("{tinyvecwtor}");
///
/// // tinyvector can be used to initialize a normal vector or array
/// // Vec<_> should also be valid
/// let vector = tinyvecwtor.collect::<Vec<i32>>();
///
/// // To iterate over a tinyvec type
/// for i in tinyvecwtor {
///     println!("{i}");
/// }
///
/// let element = tinyvecwtor.get(20);
///
/// // get() gives you an Option to avoid out of bound access
/// // the same is true for `remove()`
/// if let Some(elem) = element{
///     // do something with elem
/// }
///
///
/// // tinyvec is extensive, using vector, array, and anything else that
/// // coerces into slice.
/// // Both are valid in the following example:
/// let vector = vec![1,2,3];
/// let array = [4, 5, 6];
/// tinyvecwtor.extend(&vector);
/// tinyvecwtor.extend(&array);
/// ```
#[derive(Debug)]
pub struct tinyvec<T, const N: usize>
where
    T: Copy,
    T: Default,
    T: Display,
{
    stack: [MaybeUninit<T>; N],
    heap: Vec<T>,
    counter: usize,
    iters: step_iter,
}

impl<T, const N: usize> tinyvec<T, N>
where
    T: Copy,
    T: Default + Display,
{
    /// New tinyvector
    /// with default heap capacity: `general_heap: usize`.
    pub fn new() -> Self {
        Self {
            stack: unsafe { MaybeUninit::uninit().assume_init() },
            heap: Vec::with_capacity(general_heap),
            counter: 0,
            iters: step_iter::Not,
        }
    }

    /// Return length(stack + heap). For more information
    /// use capacity().
    pub fn len(&self) -> usize {
        self.counter
    }

    /// Returns (capacity_on_stack, capacity_on_heap)
    pub fn capacity(&self) -> (usize, usize) {
        (N, self.heap.capacity())
    }

    /// Push value to stack if `counter < general`
    /// else push to heap.
    pub fn push(&mut self, element: T) {
        if self.counter >= N {
            self.heap.push(element);
        } else {
            self.stack[self.counter] = MaybeUninit::new(element);
        }

        self.counter += 1;
    }

    pub fn get(&self, at: usize) -> Option<T> {
        if at >= self.counter {
            return None;
        }

        if at < N {
            unsafe {
                return Some(*self.stack[at].as_ptr());
            }
        } else {
            return Some(self.heap[at - N]);
        }
    }

    /// Returns Option instead of `T`
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.counter {
            return None;
        }

        if index < N {
            unsafe {
                let value = self.stack[index].as_ptr().read();
                for i in index..N - 1 {
                    self.stack[i] = MaybeUninit::new(self.stack[i + 1].as_ptr().read());
                }

                self.counter -= 1;
                return Some(value);
            }
        } else {
            let value = self.heap[index];
            self.heap.remove(index - N);
            self.counter -= 1;
            return Some(value);
        }
    }

    /// Returns `T` as long as tinyvec holds some elements.
    /// If heap.pop() fails or len == 0 returns `T::default()`.
    pub fn pop(&mut self) -> T {
        if self.len() == 0 {
            return T::default();
        }

        if self.heap.is_empty() {
            unsafe {
                let value = self.stack[N - 1].as_ptr().read();
                self.stack[N - 1] = MaybeUninit::uninit();
                self.counter -= 1;
                return value;
            }
        } else {
            let value = self.heap.pop();
            self.counter -= 1;
            return value.unwrap_or(T::default());
        }
    }

    /// Extend tinyvec with a `&[T]`.
    /// Vectors, Arrays, etc. can be coerced into &T,
    /// so this is a blanket implementation for all them.
    pub fn extend(&mut self, elements: &[T]) {
        for i in elements.iter() {
            self.push(*i);
        }
    }
}

/// Make tinyvec work as:
/// ```rust
/// let mut tinyvecwtor: tinyvec<u8, 256> = tinyvec::new();
/// for i in 0..=256 {
///     tinyvecwtor.push(i as u8);
/// }
///
/// for i in tinyvecwtor {
///     // do something with i
/// }
/// ````
///
/// Iterator trait also provides free implementation of many
/// other utility function.
///
/// ```rust
/// let tinyvector: tinyvec<i32, 1024> = tinyvec::new()
/// // fill tinyvector
/// let vector: Vec<i32> = (tinyvector).collect::<Vec<i32>>();
/// ````
/// is valid.
///
impl<T, const N: usize> iter::Iterator for tinyvec<T, N>
where
    T: Default,
    T: Copy + Display,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.iters == step_iter::Not {
            self.iters = step_iter::Yes(0);
        } else {
            if let step_iter::Yes(idx) = self.iters {
                self.iters = step_iter::Yes(idx + 1);
            }
        }

        if let step_iter::Yes(at) = self.iters {
            if at <= self.counter {
                return self.get(at);
            }
        }

        None
    }
}

/// tinyvec only supports `Copy-Types` as of now.
impl<T, const N: usize> Display for tinyvec<T, N>
where
    T: Copy + Default + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut res = String::from("[ ");
        for i in 0..self.counter {
            let text = format!("{}", self.get(i).unwrap_or(T::default()));
            res.push_str(&text);

            if i == self.counter - 1 {
                res.push_str(" ");
                break;
            }
            res.push_str(", ");
        }

        res.push_str("]");
        write!(f, "{res}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{general_heap, tinyvec};

    #[test]
    fn setup() {
        let mut vector: tinyvec<i32, 1024> = tinyvec::new();
        vector.push(1);
        vector.push(2);
        vector.push(3);
        let _ = vector.pop();
        let _ = vector.remove(0);
        assert_eq!(vector.len(), 1);
        assert_eq!(vector.capacity(), (1024, 1024));
    }

    #[test]
    fn length() {
        let mut vector: tinyvec<bool, 1024> = tinyvec::new();
        let _ = vector.pop();
        assert_eq!(vector.len(), 0);
    }

    #[test]
    fn extends() {
        let mut vector: tinyvec<char, 2048> = tinyvec::new();
        let slice = ('a'..='z').collect::<Vec<char>>();
        vector.extend(&slice);
        assert_eq!(vector.len(), 26);
    }

    #[test]
    fn iterate() {
        let mut vector: tinyvec<i32, 4> = tinyvec::new();
        let slice: [i32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        vector.extend(&slice);
        assert_eq!(vector.len(), 8);
        assert_eq!(vector.capacity(), (4, general_heap));

        let mut number: i32 = 0;
        for i in vector {
            number += i;
        }

        assert_eq!(number, 36);
    }

    #[test]
    fn convert() {
        // Convert tinyvec to vec
        let mut tinyvecwtor: tinyvec<i128, 200> = tinyvec::new();
        for i in 0..200 {
            tinyvecwtor.push(i as i128);
        }

        let vector = tinyvecwtor.collect::<Vec<_>>();
        assert_eq!(vector.len(), 200);
    }
}
