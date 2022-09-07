//! Trivial type-map implementation
//!
//! Implementation uses type erased values with type as index.
//! Due to limitation of `TypeId` only types without non-static references are supported. (in future it can be changed)
//!
//! ## Hash implementation
//!
//! The map uses simplified `Hasher` that relies on fact that `TypeId` produces unique values only.
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

use core::any::TypeId;

mod hash;

type Key = TypeId;

#[cold]
#[inline(never)]
fn unlikely_vacant_insert(this: std::collections::hash_map::VacantEntry<'_, Key, ValueBox>, val: ValueBox) -> &'_ mut ValueBox {
    this.insert(val)
}

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

type HashMap = std::collections::HashMap<Key, ValueBox, hash::UniqueHasherBuilder>;

///Type-safe store, indexed by types.
pub struct TypeMap {
    inner: HashMap,
}

///Valid type for [TypeMap]
pub trait Value: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Value for T {}

///Shared reference to [Value]
pub type ValueRef<'a> = &'a (dyn core::any::Any + Send + Sync);
///Mutable reference to [Value]
pub type ValueMut<'a> = &'a mut (dyn core::any::Any + Send + Sync);
///Boxed [Value]
pub type ValueBox = Box<dyn core::any::Any + Send + Sync>;

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
    pub fn has<T: Value>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    ///Returns whether element is present in the map.
    pub fn contains_key<T: Value>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    ///Access element in the map, returning reference to it, if present
    pub fn get<T: Value>(&self) -> Option<&T> {
        match self.inner.get(&TypeId::of::<T>()) {
            Some(ptr) => match ptr.downcast_ref() {
                Some(res) => Some(res),
                None => unreach!(),
            },
            None => None
        }
    }

    #[inline]
    ///Access element in the map with type-id provided at runtime, returning reference to it, if present
    pub fn get_raw(&self, k: TypeId) -> Option<ValueRef> {
        match self.inner.get(&k) {
            Some(ptr) => Some(ptr.as_ref()),
            None => None
        }
    }

    #[inline]
    ///Access element in the map, returning mutable reference to it, if present
    pub fn get_mut<T: Value>(&mut self) -> Option<&mut T> {
        match self.inner.get_mut(&TypeId::of::<T>()) {
            Some(ptr) => match ptr.downcast_mut() {
                Some(res) => Some(res),
                None => unreach!(),
            },
            None => None
        }
    }

    #[inline]
    ///Access element in the map with type-id provided at runtime, returning mutable reference to it, if present
    pub fn get_mut_raw(&mut self, k: TypeId) -> Option<ValueMut> {
        match self.inner.get_mut(&k) {
            Some(ptr) => Some(ptr.as_mut()),
            None => None
        }
    }

    #[inline]
    ///Access element in the map, if not present, constructs it using default value.
    pub fn get_or_default<T: Value + Default>(&mut self) -> &mut T {
        use std::collections::hash_map::Entry;

        match self.inner.entry(TypeId::of::<T>()) {
            Entry::Occupied(occupied) => {
                match occupied.into_mut().downcast_mut() {
                    Some(res) => res,
                    None => unreach!(),
                }
            },
            Entry::Vacant(vacant) => {
                let ptr = unlikely_vacant_insert(vacant, Box::new(T::default()));
                match ptr.downcast_mut() {
                    Some(res) => res,
                    None => unreach!(),
                }
            }
        }
    }

    ///Insert element inside the map, returning heap-allocated old one if any
    ///
    ///## Note
    ///
    ///Be careful when inserting without explicitly specifying type.
    ///Some special types like function pointers are impossible to infer as non-anonymous type.
    ///You should manually specify type when in doubt.
    pub fn insert<T: Value>(&mut self, value: T) -> Option<Box<T>> {
        use std::collections::hash_map::Entry;

        match self.inner.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut occupied) => {
                let result = occupied.insert(Box::new(value));
                match result.downcast() {
                    Ok(result) => Some(result),
                    Err(_) => unreach!()
                }
            },
            Entry::Vacant(vacant) => {
                vacant.insert(Box::new(value));
                None
            }
        }
    }

    ///Insert boxed element inside the map with dynamic type,
    ///returning heap-allocated old one with the same type-id if any.
    ///
    ///This does not reallocate `value`.
    pub fn insert_raw(&mut self, value: ValueBox) -> Option<ValueBox> {
        use std::collections::hash_map::Entry;

        match self.inner.entry(value.as_ref().type_id()) {
            Entry::Occupied(mut occupied) => {
                let result = occupied.insert(value);
                Some(result)
            },
            Entry::Vacant(vacant) => {
                vacant.insert(value);
                None
            }
        }
    }

    ///Attempts to remove element from the map, returning boxed `Some` if it is present.
    pub fn remove<T: Value>(&mut self) -> Option<Box<T>> {
        self.inner.remove(&TypeId::of::<T>()).map(|ptr| {
            match ptr.downcast() {
                Ok(result) => result,
                Err(_) => unreach!()
            }
        })
    }

    #[inline]
    ///Attempts to remove element from the map with type-id provided at runtime, returning boxed `Some` if it is present.
    pub fn remove_raw(&mut self, id: TypeId) -> Option<ValueBox> {
        self.inner.remove(&id)
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