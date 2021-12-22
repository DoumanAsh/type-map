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

type HashMap = std::collections::HashMap<TypeId, Box<dyn core::any::Any + Send + Sync>, hash::UniqueHasherBuilder>;

///Type-safe store, indexed by types.
pub struct TypeMap {
    inner: HashMap,
}

///Valid type for `TypeMap`
pub trait Type: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Type for T {}

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
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    ///Returns whether element is present in the map.
    pub fn contains_key<T: Type>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    ///Access element in the map, returning reference to it, if present
    pub fn get<T: Type>(&self) -> Option<&T> {
        match self.inner.get(&TypeId::of::<T>()) {
            Some(ptr) => match ptr.downcast_ref() {
                Some(res) => Some(res),
                None => unreach!(),
            },
            None => None
        }
    }

    #[inline]
    ///Access element in the map, returning mutable reference to it, if present
    pub fn get_mut<T: Type>(&mut self) -> Option<&mut T> {
        match self.inner.get_mut(&TypeId::of::<T>()) {
            Some(ptr) => match ptr.downcast_mut() {
                Some(res) => Some(res),
                None => unreach!(),
            },
            None => None
        }
    }

    #[inline]
    ///Access element in the map, if not present, constructs it using default value.
    pub fn get_or_default<T: Type + Default>(&mut self) -> &mut T {
        use std::collections::hash_map::Entry;

        match self.inner.entry(TypeId::of::<T>()) {
            Entry::Occupied(occupied) => {
                match occupied.into_mut().downcast_mut() {
                    Some(res) => res,
                    None => unreach!(),
                }
            },
            Entry::Vacant(vacant) => {
                let ptr = vacant.insert(Box::new(T::default()));
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
    pub fn insert<T: Type>(&mut self, value: T) -> Option<Box<T>> {
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

    ///Attempts to remove element from the map, returning boxed `Some` if it is present.
    pub fn remove<T: Type>(&mut self) -> Option<Box<T>> {
        self.inner.remove(&TypeId::of::<T>()).map(|ptr| {
            match ptr.downcast() {
                Ok(result) => result,
                Err(_) => unreach!()
            }
        })
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
