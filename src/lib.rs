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
type Value = Box<dyn core::any::Any + Send + Sync>;

#[cold]
#[inline(never)]
fn unlikely_vacant_insert(this: std::collections::hash_map::VacantEntry<'_, Key, Value>, val: Value) -> &'_ mut Value {
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

type HashMap = std::collections::HashMap<Key, Value, hash::UniqueHasherBuilder>;

///Type-safe store, indexed by types.
pub struct TypeMap {
    inner: HashMap,
}

///Valid type for `TypeMap`
pub trait Type: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Type for T {}

///Boxed dynamically-typed value
#[repr(transparent)]
pub struct TypeBox(pub Box<dyn core::any::Any + Send + Sync>);

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

    ///Insert element inside the map, returning heap-allocated old one if any
    pub fn insert_box(&mut self, value: TypeBox) -> Option<TypeBox> {
        use std::collections::hash_map::Entry;

        match self.inner.entry(value.boxed_type_id()) {
            Entry::Occupied(mut occupied) => {
                let result = occupied.insert(value.0);
                match result.downcast() {
                    Ok(result) => Some(*result),
                    Err(_) => unreach!()
                }
            },
            Entry::Vacant(vacant) => {
                vacant.insert(value.0);
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

    #[inline]
    ///Attempts to remove element from the map with the given id, returning `TypeBox` if it is present.
    pub fn remove_box(&mut self, id: TypeId) -> Option<TypeBox> {
        self.inner.remove(&id).map(TypeBox)
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

impl TypeBox {
    /// Allocates and stores the given value on the heap
    pub fn new<T: Type>(value: T) -> Self {
        Self(Box::new(value))
    }

    /// Attempts to downcast to the given type.
    ///
    /// If successful, consumes and returns the boxed value.
    /// If unsuccessful, returns `self` unmodified.
    pub fn into_inner_downcast<T: Type>(self) -> Result<T, Self> {
        match self.0.downcast() {
            Ok(inner) => Ok(*inner),
            Err(inner) => Err(Self(inner))
        }
    }

    /// Gets the type id of the boxed value.
    pub fn boxed_type_id(&self) -> TypeId {
        use core::any::Any;

        (&self.0).type_id()
    }
}