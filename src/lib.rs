use std::cell::Cell;
use std::fmt;

pub struct CellOpt<T> {
    slot: Cell<Option<T>>,
}

impl<T> Default for CellOpt<T> {
    fn default() -> Self {
        CellOpt {
            slot: Cell::new(None),
        }
    }
}

impl<T: Clone> Clone for CellOpt<T> {
    fn clone(&self) -> Self {
        self.apply_then_restore(|inner| CellOpt::new(inner.clone()))
            .unwrap_or_default()
    }
}

impl<T: fmt::Debug> fmt::Debug for CellOpt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        self.apply_then_restore(|inner| write!(f, "{}", format_args!("Option::Some({:?})", inner)))
            .unwrap_or_else(|| write!(f, "None"))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Occupied,
    Empty,
}

pub struct InsertErr<T> {
    pub insert_try: T,
    pub err: Error,
}

impl<T> CellOpt<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            slot: Cell::new(value.into()),
        }
    }

    #[inline]
    pub fn apply_then_restore<U, F: FnMut(&T) -> U>(&self, mut f: F) -> Option<U> {
        self.take()
            .map(|t| {
                let u = f(&t);
                self.overwrite(t);
                u
            })
            .ok()
    }

    #[inline]
    pub fn apply_and_update<F: Fn(T) -> T>(&self, f: F) {
        if let Ok(t) = self.take() {
            self.overwrite(f(t));
        }
    }

    #[inline]
    pub fn insert(&self, value: T) -> Result<(), InsertErr<T>> {
        if self.is_occupied() {
            Err(InsertErr {
                insert_try: value,
                err: Error::Occupied,
            })
        } else {
            self.overwrite(value);
            Ok(())
        }
    }

    #[inline]
    pub fn force_take(&self) -> T {
        self.take().unwrap()
    }

    #[inline]
    pub fn take(&self) -> Result<T, Error> {
        self.slot.take().ok_or(Error::Empty)
    }

    #[inline]
    pub fn is_occupied(&self) -> bool {
        if let Ok(value) = self.take() {
            self.overwrite(value);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn overwrite(&self, value: T) {
        self.slot.replace(Some(value));
    }

    #[inline]
    pub fn clone_inner(&self) -> Option<T>
    where
        T: Clone,
    {
        self.apply_then_restore(|inner| inner.clone())
    }
}
