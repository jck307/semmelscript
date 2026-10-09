use crate::{
    Result,
};

use std::fmt::Debug;

pub struct Buffer<T> {
    pub buffer: Box<[T]>,
    pub i: usize,
}

const EOF: &'static str = "unexpected end of file";

impl<T: PartialEq + Clone + Debug> Buffer<T> {
    pub fn new(buffer: Vec<T>) -> Self {
        Self {
            buffer: buffer.into_boxed_slice(),
            i: 0,
        }
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        self.buffer.get(i)
    }

    pub fn step(&mut self) {
        self.i += 1;
    }

    pub fn stepn(&mut self, count: usize) {
        self.i += count;
    }

    pub fn back(&mut self) {
        self.i -= 1;
    }

    pub fn next(&mut self) -> Result<&T> {
        self.step();
        self.buffer.get(self.i - 1).ok_or(EOF.into())
    }

    pub fn peek(&mut self) -> Result<&T> {
        self.buffer.get(self.i).ok_or(EOF.into())
    }

    pub fn expect(&mut self, value: &T) -> Result<()> {
        let next = self.next()?;
        if next != value {
            Err(format!("expected {value:?} (found {next:?})").into())
        } else {
            Ok(())
        }
    }

    pub fn peek_from(&mut self, set: &[T]) -> Vec<T> {
        let mut result = Vec::new();
        let mut i = self.i;
        loop {
            if let Some(peek) = self.buffer.get(i) {
                if set.contains(&peek) {
                    result.push(peek.clone());
                    i += 1;
                } else {
                    break
                }
            } else {
                break
            }
        }
        result
    }

    pub fn next_from(&mut self, set: &[T]) -> Vec<T> {
        let mut result = Vec::new();
        loop {
            if let Ok(peek) = self.peek() {
                if set.contains(&peek) {
                    result.push(peek.clone());
                    self.step();
                } else {
                    break
                }
            } else {
                break
            }
        }
        result
    }
}
