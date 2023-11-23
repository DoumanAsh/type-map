use crate::{Type, ValueBox};

use core::marker::PhantomData;

#[repr(transparent)]
///Value type
pub struct Value<T> {
    inner: ValueBox,
    _typ: PhantomData<T>
}

impl<T: Type> Value<T> {
    #[inline(always)]
    ///Creates new raw Value trusting user to specify correct type
    pub unsafe fn new(inner: ValueBox) -> Self {
        Self::new_inner(inner)
    }

    #[inline(always)]
    pub(crate) fn new_inner(inner: ValueBox) -> Self {
        Self {
            inner,
            _typ: PhantomData,
        }
    }

    #[inline(always)]
    pub(crate) fn new_inner_ref(inner: &ValueBox) -> &Self {
        unsafe {
            core::mem::transmute(inner)
        }
    }

    #[inline(always)]
    pub(crate) fn new_inner_mut(inner: &mut ValueBox) -> &mut Self {
        unsafe {
            core::mem::transmute(inner)
        }
    }

    #[inline(always)]
    ///Creates instance from concrete type
    pub fn from_boxed(inner: Box<T>) -> Self {
        Self::new_inner(inner)
    }

    #[inline]
    ///Downcasts self into concrete type
    pub fn downcast(self) -> Box<T> {
        match self.inner.downcast() {
            Ok(res) => res,
            Err(_) => unreach!(),
        }
    }

    #[inline]
    ///Downcasts self into concrete type
    pub fn downcast_ref(&self) -> &T {
        match self.inner.downcast_ref() {
            Some(res) => res,
            None => unreach!(),
        }
    }

    #[inline]
    ///Downcasts self into concrete type
    pub fn downcast_mut(&mut self) -> &mut T {
        match self.inner.downcast_mut() {
            Some(res) => res,
            None => unreach!(),
        }
    }

    #[inline(always)]
    ///Access underlying untyped pointer
    pub fn as_raw(&self) -> &ValueBox {
        &self.inner
    }

    #[inline(always)]
    ///Access underlying untyped pointer
    pub fn as_raw_mut(&mut self) -> &mut ValueBox {
        &mut self.inner
    }


    #[inline(always)]
    ///Access underlying untyped pointer
    pub fn into_raw(self) -> ValueBox {
        self.inner
    }
}

impl<T: Type> AsRef<T> for Value<T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        self.downcast_ref()
    }
}

impl<T: Type> AsMut<T> for Value<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        self.downcast_mut()
    }
}

impl<T: Type> core::ops::Deref for Value<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.downcast_ref()
    }
}

impl<T: Type> core::ops::DerefMut for Value<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.downcast_mut()
    }
}

impl<T: Type> Into<ValueBox> for Value<T> {
    #[inline(always)]
    fn into(self) -> ValueBox {
        self.into_raw()
    }
}
