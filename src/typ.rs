use core::any::TypeId;

///Valid type allowed as key of type map
pub trait Type: 'static + Send + Sync {
    #[doc(hidden)]
    #[inline(always)]
    ///Return type id
    fn id() -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<T: 'static + Send + Sync> Type for T {}

///Tag to indicate Raw boxed value
pub struct RawType;
