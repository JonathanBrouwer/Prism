use std::iter;
use std::marker::PhantomData;

mod impls;

pub trait ExhaustiveArbitrary {
    fn arbitrary(u: &mut DataSourceTaker) -> Self;

    fn iter_exhaustive(max_length: usize) -> impl Iterator<Item=Self> where Self: Sized {
        let mut source = DataSource::new(max_length);
        iter::from_fn(move || source.next_run().map(|mut u| Self::arbitrary(&mut u)))
    }
}

pub struct DataSourceTaker<'a> {
    len_left: usize,
    buffer_data: &'a mut Vec<usize>,
    buffer_data_max: &'a mut Vec<usize>,
    buffer_idx: usize,
}

impl<'a> DataSourceTaker<'a> {
    pub fn choice(&mut self, range: usize) -> usize {
        assert!(range > 0);
        if range == 1 {
            return 0
        }

        self.len_left = self.len_left.saturating_sub(range - 1);

        if self.buffer_idx < self.buffer_data.len() {
            debug_assert_eq!(range - 1, self.buffer_data_max[self.buffer_idx]);
            let data = self.buffer_data[self.buffer_idx];
            self.buffer_idx += 1;
            return data
        }

        self.buffer_data.push(0);
        self.buffer_data_max.push(range - 1);
        self.buffer_idx += 1;
        0
    }

    pub fn len_left(&self) -> usize {
        self.len_left
    }

    pub fn take_len(&mut self, len: usize) -> Result<(), ()> {
        if self.len_left >= len {
            self.len_left -= len;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn take_any_len(&mut self) -> usize {
        let len_left = self.len_left;
        let len = self.choice(len_left + 1);
        self.len_left += len_left;
        self.len_left -= len;
        len
    }

    pub fn iter_of<'b, T: ExhaustiveArbitrary>(&'b mut self) -> DataSourceTakerIter<'a, 'b, T> {
        let len = self.take_any_len();
        DataSourceTakerIter {
            taker: self,
            left: len,
            phantom: Default::default(),
        }
    }

    pub fn recurse<T: ExhaustiveArbitrary>(&mut self, base: impl FnOnce(&mut Self) -> T, recurse: impl FnOnce(&mut Self) -> T) -> T {
        if self.len_left() > 0 && bool::arbitrary(self) {
            recurse(self)
        } else {
            base(self)
        }
    }
}

pub struct DataSourceTakerIter<'a, 'b, T: ExhaustiveArbitrary> {
    taker: &'b mut DataSourceTaker<'a>,
    left: usize,
    phantom: PhantomData<T>,
}

impl<T: ExhaustiveArbitrary> Iterator for DataSourceTakerIter<'_, '_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left > 0 {
            self.left -= 1;
            Some(T::arbitrary(self.taker))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.left, Some(self.left))
    }
}

pub struct DataSource {
    max_len: usize,
    buffer_data: Vec<usize>,
    buffer_data_max: Vec<usize>,
    first_run: bool
}

impl DataSource {
    pub fn new(max_len: usize) -> Self {
        Self {
            max_len,
            buffer_data: vec![],
            buffer_data_max: vec![],
            first_run: true
        }
    }

    pub fn next_run(&mut self) -> Option<DataSourceTaker> {
        if !self.first_run {
            for i in (0..self.buffer_data.len()).rev() {
                if self.buffer_data[i] == self.buffer_data_max[i] {
                    self.buffer_data.pop();
                    self.buffer_data_max.pop();
                } else {
                    self.buffer_data[i] += 1;
                    break;
                }
            }
            if self.buffer_data.len() == 0 {
                return None
            }
        }
        self.first_run = false;
        Some(DataSourceTaker {
            len_left: self.max_len,
            buffer_data: &mut self.buffer_data,
            buffer_data_max: &mut self.buffer_data_max,
            buffer_idx: 0,
        })
    }
}
