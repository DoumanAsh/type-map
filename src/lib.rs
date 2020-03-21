//! Trivial type-map implementation
//!
//! Implementation uses type erased values with type as index.
//! Due to limitation of `TypeId` only types without non-static references are supported. (in future it can be changed)
//!
//! ## Type erasure
//!
//! Each inserted value is stored on heap, with type erased pointer, using type as key.
//! When value is retrieved, type information is used as key and pointer is casted to corresponding the type.
//! This is safe, because Rust allows cast back and forth between pointers as long as the pointer actually points to the type (which is the case).
//!
//! Static references are allowed, but in current implementation are stored on heap.
//! It might be changed in future.
//!
//! ## Hash implementation
//!
//! The map uses simplified `Hasher` that relies on fact that `TypeId` produces unique values only.
//! In fact there is no hashing under hood, and type's id is returned as it is.
//!
//! ## Requirements:
//!
//! - `alloc` with enabled global allocator
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
#![no_std]

extern crate alloc;

use smart_ptr::unique::Unique;

use core::any::TypeId;
use core::{mem};

mod hash;

type HashMap = indexmap::IndexMap<TypeId, Unique<u8, fn (*mut u8)>, hash::UniqueHasherBuilder>;

///Type-safe store, indexed by types.
pub struct TypeMap {
    inner: HashMap,
}

///Type's map `Key` trait.
///
///Due to limitations of `TypeId`, safe code allows only to insert static types (i.e. types that doesn't contain non-static lifetimes)
pub trait Key: 'static {}
//TODO: It should be fairly trivial to replace TypeId with custom impl that allows non-statics, but
//I doubt it is good idea

impl<T: 'static> Key for T {}

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
    pub fn has<T: Key>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    ///Returns whether element is present in the map.
    pub fn contains_key<T: Key>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    ///Access element in the map, returning reference to it, if present
    pub fn get<T: Key>(&self) -> Option<&T> {
        self.inner.get(&TypeId::of::<T>()).map(|ptr| unsafe {
            &*ptr.const_cast::<T>()
        })
    }

    #[inline]
    ///Access element in the map, returning mutable reference to it, if present
    pub fn get_mut<T: Key>(&mut self) -> Option<&mut T> {
        self.inner.get_mut(&TypeId::of::<T>()).map(|ptr| unsafe {
            &mut *ptr.cast::<T>()
        })
    }

    #[inline]
    ///Access element in the map, if not present, constructs it using default value.
    pub fn get_or_default<T: Key + Default>(&mut self) -> &mut T {
        match self.inner.entry(TypeId::of::<T>()) {
            indexmap::map::Entry::Occupied(mut occupied) => unsafe {
                &mut *occupied.get_mut().cast::<T>()
            },
            indexmap::map::Entry::Vacant(vacant) => {
                let deleter: fn (*mut u8) = smart_ptr::boxed_deleter::<T>;
                let ptr = unsafe {
                    Unique::from_ptr_unchecked(alloc::boxed::Box::into_raw(alloc::boxed::Box::new(T::default())) as *mut u8, deleter)
                };

                let ptr = vacant.insert(ptr);
                unsafe {
                    &mut *ptr.cast::<T>()
                }
            }
        }
    }

    ///Insert element inside the map, returning old one if any
    ///
    ///## Note
    ///
    ///Be careful when inserting without explicitly specifying type.
    ///Some special types like function pointers are impossible to infer as non-anonymous type.
    ///You should manually specify type when in doubt.
    pub fn insert<T: Key>(&mut self, mut value: T) -> Option<T> {
        match self.inner.entry(TypeId::of::<T>()) {
            indexmap::map::Entry::Occupied(mut occupied) => {
                mem::swap(unsafe { &mut *occupied.get_mut().cast::<T>() }, &mut value);
                Some(value)
            },
            indexmap::map::Entry::Vacant(vacant) => {
                let deleter: fn (*mut u8) = smart_ptr::boxed_deleter::<T>;
                let ptr = unsafe {
                    Unique::from_ptr_unchecked(alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value)) as *mut u8, deleter)
                };

                vacant.insert(ptr);
                None
            }
        }
    }

    ///Attempts to remove element from the map, returning `Some` if it is present
    pub fn remove<T: Key>(&mut self) -> Option<T> {
        self.inner.remove(&TypeId::of::<T>()).map(|ptr| unsafe {
            let ptr = alloc::boxed::Box::from_raw(ptr.release().as_ptr() as *mut T);
            *ptr
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
