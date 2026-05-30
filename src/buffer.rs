use crate::{
    Result,
};

use std::fmt::Debug;

pub struct Buffer<T> {
    pub(crate) buffer: Box<[T]>,
    pub(crate) i: usize,
}

const EOF: &'static str = "unexpected end of file";

impl<T: PartialEq + Clone + Debug> Buffer<T> {
    pub(crate) fn new(buffer: Vec<T>) -> Self {
        Self {
            buffer: buffer.into_boxed_slice(),
            i: 0,
        }
    }

    pub(crate) fn get(&self, i: usize) -> Option<&T> {
        self.buffer.get(i)
    }

    pub(crate) fn step(&mut self) {
        self.i += 1;
    }

    pub(crate) fn stepn(&mut self, amount: usize) {
        self.i += amount;
    }

    pub(crate) fn back(&mut self) {
        self.i -= 1;
    }

    pub(crate) fn next(&mut self) -> Result<&T> {
        self.step();
        self.buffer.get(self.i - 1).ok_or(EOF.into())
    }

    pub(crate) fn peek(&mut self) -> Result<&T> {
        self.buffer.get(self.i).ok_or(EOF.into())
    }

    pub(crate) fn expect(&mut self, value: &T) -> Result<()> {
        let next = self.next()?;
        if next != value {
            Err(format!("expected {value:?} (found {next:?})").into())
        } else {
            Ok(())
        }
    }

    pub(crate) fn peek_from(&mut self, set: &[T]) -> Vec<T> {
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

    pub(crate) fn next_from(&mut self, set: &[T]) -> Vec<T> {
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
