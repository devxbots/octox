use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>;

#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }
}

#[derive(Debug, Default)]
pub struct State {
    /// Type-based store
    ///
    /// The implementation for this type-based map is inspired by the `Extensions` store in the
    /// [`http`](https://github.com/hyperium/http) crate.
    store: Box<AnyMap>,
}

impl State {
    pub fn new() -> Self {
        Self {
            store: Box::new(HashMap::default()),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.store
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|boxed| *boxed)
            })
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.store
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.store
            .as_mut()
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn state_stores_and_returns_value() {
        let mut state = State::new();

        state.insert(64u32);

        assert_eq!(Some(&64), state.get::<u32>());
    }

    #[test]
    fn state_returns_none_when_value_is_missing() {
        let mut state = State::new();

        state.insert(64u32);

        assert_eq!(None, state.get::<i32>());
    }
}
