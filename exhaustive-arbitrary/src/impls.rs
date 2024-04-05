use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::{BinaryHeap, BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use crate::{ChoiceError, DataSourceTaker, ExhaustiveArbitrary};

impl ExhaustiveArbitrary for bool {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.choice(2)? != 0)
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for Box<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for Rc<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for Arc<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for Cell<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for RefCell<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for UnsafeCell<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for Mutex<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for RwLock<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(Self::new(T::arbitrary(u)?))
    }
}

impl ExhaustiveArbitrary for () {
    fn arbitrary(_: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(())
    }
}

impl<T1: ExhaustiveArbitrary> ExhaustiveArbitrary for (T1,) {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok((T1::arbitrary(u)?,))
    }
}

impl<T1: ExhaustiveArbitrary, T2: ExhaustiveArbitrary> ExhaustiveArbitrary for (T1,T2) {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok((T1::arbitrary(u)?,T2::arbitrary(u)?,))
    }
}

impl<T1: ExhaustiveArbitrary, T2: ExhaustiveArbitrary, T3: ExhaustiveArbitrary> ExhaustiveArbitrary for (T1,T2,T3) {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok((T1::arbitrary(u)?,T2::arbitrary(u)?,T3::arbitrary(u)?,))
    }
}

impl<T1: ExhaustiveArbitrary, T2: ExhaustiveArbitrary, T3: ExhaustiveArbitrary, T4: ExhaustiveArbitrary> ExhaustiveArbitrary for (T1,T2,T3,T4) {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok((T1::arbitrary(u)?,T2::arbitrary(u)?,T3::arbitrary(u)?,T4::arbitrary(u)?))
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for Option<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        if bool::arbitrary(u)? {
            Ok(None)
        } else {
            Ok(Some(T::arbitrary(u)?))
        }
    }
}

impl<T: ExhaustiveArbitrary, E: ExhaustiveArbitrary> ExhaustiveArbitrary for Result<T, E> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        if bool::arbitrary(u)? {
            Ok(Ok(T::arbitrary(u)?))
        } else {
            Ok(Err(E::arbitrary(u)?))
        }
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for Vec<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<T>()?.collect::<Result<_, _>>()?)
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for LinkedList<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<T>()?.collect::<Result<_, _>>()?)
    }
}

impl<T: ExhaustiveArbitrary> ExhaustiveArbitrary for VecDeque<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<T>()?.collect::<Result<_, _>>()?)
    }
}


impl<T: ExhaustiveArbitrary + Ord> ExhaustiveArbitrary for BTreeSet<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<T>()?.collect::<Result<_, _>>()?)
    }
}

impl<K: ExhaustiveArbitrary + Ord, V: ExhaustiveArbitrary> ExhaustiveArbitrary for BTreeMap<K, V> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<(K, V)>()?.collect::<Result<_, _>>()?)
    }
}

impl<T: ExhaustiveArbitrary + Hash + Eq> ExhaustiveArbitrary for HashSet<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<T>()?.collect::<Result<_, _>>()?)
    }
}

impl<K: ExhaustiveArbitrary + Hash + Eq, V: ExhaustiveArbitrary> ExhaustiveArbitrary for HashMap<K, V> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<(K, V)>()?.collect::<Result<_, _>>()?)
    }
}

impl<T: ExhaustiveArbitrary + Ord> ExhaustiveArbitrary for BinaryHeap<T> {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok(u.iter_of::<T>()?.collect::<Result<_, _>>()?)
    }
}

impl<const N: usize, T: ExhaustiveArbitrary> ExhaustiveArbitrary for [T; N] {
    fn arbitrary(u: &mut DataSourceTaker) -> Result<Self, ChoiceError> {
        Ok((0..N).map(|_| T::arbitrary(u)).collect::<Result<Vec<_>, _>>()?.try_into().unwrap_or_else(|_| unreachable!()))
    }
}


