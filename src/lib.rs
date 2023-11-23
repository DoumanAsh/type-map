//! Trivial type-map implementation
//!
//! Implementation uses type erased values with type as index.
//!
//! ## Hash implementation
//!
//! The map uses simplified `Hasher` that relies on fact that `Type::id` is unique.
//! In fact there is no hashing under hood, and type's id is returned as it is.
//!
//! ## Usage
//!
//! ```rust
//! use ttmap::TypeMap;
//!
//! let mut map = TypeMap::new();
//!
//! map.insert("string");
//!
//! assert_eq!(*map.get::<&'static str>().unwrap(), "string");
//!
//! map.insert(1u8);
//!
//! assert_eq!(*map.get::<u8>().unwrap(), 1);
//!
//! assert_eq!(map.get_or_default::<String>(), "");
//! ```

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(not(debug_assertions))]
macro_rules! unreach {
    () => ({
        unsafe {
            core::hint::unreachable_unchecked();
        }
    })
}

#[cfg(debug_assertions)]
macro_rules! unreach {
    () => ({
        unreachable!()
    })
}

mod typ;
pub use typ::Type;
mod value;
pub use value::Value;
mod hash;

type Key = core::any::TypeId;
///Boxed [Type]
pub type ValueBox = Box<dyn core::any::Any + Send + Sync>;

#[cold]
#[inline(never)]
fn unlikely_vacant_insert(this: std::collections::hash_map::VacantEntry<'_, Key, ValueBox>, val: ValueBox) -> &'_ mut ValueBox {
    this.insert(val)
}

type HashMap = std::collections::HashMap<Key, ValueBox, hash::UniqueHasherBuilder>;

///Type-safe store, indexed by types.
pub struct TypeMap {
    inner: HashMap,
}

impl TypeMap {
    #[inline]
    ///Creates new instance
    pub fn new() -> Self {
        Self {
            inner: HashMap::with_capacity_and_hasher(0, hash::UniqueHasherBuilder),
        }
    }

    #[inline]
    ///Returns number of key & value pairs inside.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    ///Returns number of key & value pairs inside.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    ///Returns whether map is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    ///Removes all pairs of key & value from the map.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    #[inline]
    ///Returns whether element is present in the map.
    pub fn has<T: Type>(&self) -> bool {
        self.inner.contains_key(&T::id())
    }

    #[inline]
    ///Returns whether element is present in the map.
    pub fn contains_key<T: Type>(&self) -> bool {
        self.inner.contains_key(&T::id())
    }

    #[inline]
    ///Access element in the map, returning reference to it, if present
    pub fn get<T: Type>(&self) -> Option<&T> {
        self.get_raw::<T>().map(Value::downcast_ref)
    }

    #[inline]
    ///Access element in the map, returning reference to it, if present
    pub fn get_raw<T: Type>(&self) -> Option<&Value<T>> {
        self.inner.get(&T::id()).map(Value::new_inner_ref)
    }

    #[inline]
    ///Access element in the map, returning mutable reference to it, if present
    pub fn get_mut<T: Type>(&mut self) -> Option<&mut T> {
        self.get_mut_raw::<T>().map(Value::downcast_mut)
    }

    #[inline]
    ///Access element in the map, returning mutable reference to it, if present
    pub fn get_mut_raw<T: Type>(&mut self) -> Option<&mut Value<T>> {
        self.inner.get_mut(&T::id()).map(Value::new_inner_mut)
    }

    #[inline]
    ///Access element in the map, if not present, constructs it using default value.
    pub fn get_or_default<T: Type + Default>(&mut self) -> &mut T {
        use std::collections::hash_map::Entry;

        match self.inner.entry(T::id()) {
            Entry::Occupied(occupied) => {
                match occupied.into_mut().downcast_mut() {
                    Some(res) => res,
                    None => unreach!(),
                }
            },
            Entry::Vacant(vacant) => {
                let ptr = unlikely_vacant_insert(vacant, Box::<T>::default());
                match ptr.downcast_mut() {
                    Some(res) => res,
                    None => unreach!(),
                }
            }
        }
    }

    #[inline]
    ///Insert element inside the map, returning heap-allocated old one if any
    ///
    ///## Note
    ///
    ///Be careful when inserting without explicitly specifying type.
    ///Some special types like function pointers are impossible to infer as non-anonymous type.
    ///You should manually specify type when in doubt.
    pub fn insert<T: Type>(&mut self, value: T) -> Option<Box<T>> {
        self.insert_raw(Value::new_inner(Box::new(value))).map(Value::downcast)
    }

    ///Insert raw element inside the map, returning heap-allocated old one if any
    pub fn insert_raw<T: Type>(&mut self, value: Value<T>) -> Option<Value<T>> {
        use std::collections::hash_map::Entry;

        match self.inner.entry(T::id()) {
            Entry::Occupied(mut occupied) => Some(
                Value::<T>::new_inner(
                    occupied.insert(value.into_raw())
                )
            ),
            Entry::Vacant(vacant) => {
                vacant.insert(value.into_raw());
                None
            }
        }
    }

    #[inline]
    ///Attempts to remove element from the map, returning boxed `Some` if it is present.
    pub fn remove_raw<T: Type>(&mut self) -> Option<Value<T>> {
        self.inner.remove(&T::id()).map(Value::new_inner)
    }

    #[inline]
    ///Attempts to remove element from the map, returning boxed `Some` if it is present.
    pub fn remove<T: Type>(&mut self) -> Option<Box<T>> {
        self.inner.remove(&T::id()).map(|val| Value::<T>::new_inner(val).downcast())
    }
}

impl core::default::Default for TypeMap {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for TypeMap {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "TypeMap {{ size={}, capacity={} }}", self.len(), self.capacity())
    }
}
