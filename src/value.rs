use crate::typ::{Type, RawType};
use crate::ValueBox;

use core::marker::PhantomData;

#[repr(transparent)]
///Value type
pub struct Value<T> {
    inner: ValueBox,
    _typ: PhantomData<T>
}

impl Value<RawType> {
    #[inline]
    ///Attempts downcast self into specified type
    pub fn try_downcast<O: Type>(self) -> Result<Box<O>, Self> {
        match self.inner.downcast() {
            Ok(res) => Ok(res),
            Err(inner) => Err(Self::new_inner(inner)),
        }
    }

    #[inline]
    ///Attempts to downcast self into concrete type
    pub fn try_downcast_ref<O: Type>(&self) -> Option<&O> {
        self.inner.downcast_ref()
    }

    #[inline]
    ///Attempts to downcast self into concrete type
    pub fn try_downcast_mut<O: Type>(&mut self) -> Option<&mut O> {
        self.inner.downcast_mut()
    }
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
        //dum dum no specialization
        if T::id() == RawType::id() {
            panic!("Raw box cannot use this method")
        }

        match self.inner.downcast() {
            Ok(res) => res,
            Err(_) => unreach!(),
        }
    }


    #[inline]
    ///Downcasts self into concrete type
    pub fn downcast_ref(&self) -> &T {
        //dum dum no specialization
        if T::id() == RawType::id() {
            panic!("Raw box cannot use this method")
        }

        match self.inner.downcast_ref() {
            Some(res) => res,
            None => unreach!(),
        }
    }

    #[inline]
    ///Downcasts self into concrete type
    pub fn downcast_mut(&mut self) -> &mut T {
        //dum dum no specialization
        if T::id() == RawType::id() {
            panic!("Raw box cannot use this method")
        }

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
