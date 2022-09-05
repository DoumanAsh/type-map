use std::any::TypeId;
use ttmap::{TypeBox, TypeMap};

#[test]
fn check_type_map() {
    let mut map = TypeMap::new();

    assert!(map.is_empty());
    assert_eq!(map.len(), 0);

    assert!(map.insert("test").is_none());
    assert_eq!(*map.insert("lolka").unwrap(), "test");
    assert_eq!(*map.get::<&'static str>().unwrap(), "lolka");

    assert!(map.insert::<fn ()>(check_type_map).is_none());
    assert!(*map.get::<fn ()>().unwrap() == check_type_map);

    assert!(!map.has::<usize>());
    assert_eq!(*map.get_or_default::<usize>(), 0);
    *map.get_or_default::<usize>() = 5;
    assert_eq!(*map.get_or_default::<usize>(), 5);

    assert_eq!(*map.get::<usize>().unwrap(), 5);

    assert!(!map.is_empty());
    assert_eq!(map.len(), 3);

    assert_eq!(*map.remove::<usize>().unwrap(), 5);
    assert_eq!(map.len(), 2);
    assert_eq!(map.remove::<usize>(), None);

    assert_eq!(*map.remove::<&'static str>().unwrap(), "lolka");
    assert_eq!(map.len(), 1);
    assert_eq!(map.remove::<&'static str>(), None);

    assert!(*map.remove::<fn ()>().unwrap() == check_type_map);
    assert_eq!(map.len(), 0);
    assert_eq!(map.remove::<fn ()>(), None);

    assert!(map.is_empty());

    map.clear();
    assert!(map.is_empty());
}

#[test]
fn check_type_box() {
    let mut map = TypeMap::new();

    assert!(map.is_empty());
    assert_eq!(map.len(), 0);

    assert!(map.insert("test").is_none());
    assert_eq!(*(*map.insert_box(TypeBox::new("lolka")).unwrap()).downcast_ref::<&'static str>().unwrap(), "test");
    assert_eq!(*map.get::<&'static str>().unwrap(), "lolka");

    let str_box = map.remove_box(TypeId::of::<&'static str>()).unwrap();
    assert!(map.remove_box(TypeId::of::<&'static str>()).is_none());
    assert_eq!(str_box.boxed_type_id(), TypeId::of::<&'static str>());
    let str_box = str_box.into_inner_downcast::<bool>().unwrap_err();
    assert_eq!(str_box.into_inner_downcast::<&'static str>().unwrap(), "lolka");
}

#[test]
fn check_dtor_called() {
    let mut is_called = false;
    struct CustomData<'a>(&'a mut bool);

    impl<'a> Drop for CustomData<'a> {
        fn drop(&mut self) {
            *self.0 = true;
        }
    }

    {
        let is_called: &'static mut bool = unsafe {
            core::mem::transmute(&mut is_called)
        };
        let data = CustomData(is_called);
        let mut map = TypeMap::new();

        assert!(map.insert(data).is_none());
    }

    assert!(is_called);
}
