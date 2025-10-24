use std::alloc::{Layout, alloc, dealloc, realloc};
use std::fmt;
use std::ptr::NonNull;
use std::slice;

pub trait Tuple {
    type Pointers: Tuple + Copy;
    const DANGLING_ITEMS: Self::Pointers;

    type Ref<'a>: Tuple + Copy
    where
        Self: 'a;

    type RefMut<'a>: Tuple
    where
        Self: 'a;

    type Slices<'a>: Tuple + Copy
    where
        Self: 'a;

    type MutSlices<'a>: Tuple
    where
        Self: 'a;

    unsafe fn ptrs_as_slices<'a>(ptrs: Self::Pointers, len: usize) -> Self::Slices<'a>;
    unsafe fn ptrs_as_mut_slices<'a>(ptrs: Self::Pointers, len: usize) -> Self::MutSlices<'a>;
    fn get<'a>(slices: Self::Slices<'a>, index: usize) -> Option<Self::Ref<'a>>;
    fn get_mut<'a>(slices: Self::MutSlices<'a>, index: usize) -> Option<Self::RefMut<'a>>;
    unsafe fn realloc_ptrs(ptrs: &mut Self::Pointers, old_cap: usize, new_cap: usize);
    unsafe fn write_items(ptrs: Self::Pointers, items: Self, index: usize);
    unsafe fn drop_items(ptrs: Self::Pointers, len: usize);
    unsafe fn dealloc_ptrs(ptrs: Self::Pointers, cap: usize);
}

macro_rules! impl_tuple {
    ($(($t:ident, $ptr:ident, $item:ident),)+) => {
        impl<$($t),+> Tuple for ($($t,)+) {
            type Pointers = ($(NonNull<$t>,)+);
            const DANGLING_ITEMS: Self::Pointers = ($(NonNull::<$t>::dangling(),)+);

            type Ref<'a> = ($(&'a $t,)+) where Self: 'a;
            type RefMut<'a> = ($(&'a mut $t,)+) where Self: 'a;

            type Slices<'a> = ($(&'a [$t],)+) where Self: 'a;
            type MutSlices<'a> = ($(&'a mut [$t],)+) where Self: 'a;

            #[inline]
            unsafe fn ptrs_as_slices<'a>(ptrs: Self::Pointers, len: usize) -> Self::Slices<'a> {
                let ($($ptr,)+) = ptrs;

                unsafe {
                    (
                        $(slice::from_raw_parts($ptr.as_ptr(), len),)+
                    )
                }
            }

            #[inline]
            unsafe fn ptrs_as_mut_slices<'a>(ptrs: Self::Pointers, len: usize) -> Self::MutSlices<'a> {
                let ($($ptr,)+) = ptrs;

                unsafe {
                    (
                        $(slice::from_raw_parts_mut($ptr.as_ptr(), len),)+
                    )
                }
            }

            #[inline]
            fn get<'a>(slices: Self::Slices<'a>, index: usize) -> Option<Self::Ref<'a>> {
                let ($($item,)+) = slices;
                Some(($($item.get(index)?,)+))
            }

            #[inline]
            fn get_mut<'a>(slices: Self::MutSlices<'a>, index: usize) -> Option<Self::RefMut<'a>> {
                let ($($item,)+) = slices;
                Some(($($item.get_mut(index)?,)+))
            }

            unsafe fn realloc_ptrs(ptrs: &mut Self::Pointers, old_cap: usize, new_cap: usize) {
                let ($($ptr,)+) = ptrs;

                $({
                    let new_layout = Layout::array::<$t>(new_cap).expect("invalid layout");

                    let new_ptr = if old_cap > 0 {
                        let old_layout = Layout::array::<$t>(old_cap).expect("invalid layout");

                        unsafe {
                            realloc(
                                $ptr.as_ptr().cast::<u8>(),
                                old_layout,
                                new_layout.size(),
                            )
                        }
                    } else {
                        unsafe {
                            alloc(new_layout)
                        }
                    };

                    *$ptr = NonNull::new(new_ptr.cast::<$t>()).expect("allocation failed");
                })+
            }

            #[inline]
            unsafe fn write_items(ptrs: Self::Pointers, items: Self, index: usize) {
                let ($($ptr,)+) = ptrs;
                let ($($item,)+) = items;

                unsafe {
                    $($ptr.as_ptr().add(index).write($item);)+
                }
            }

            unsafe fn drop_items(ptrs: Self::Pointers, len: usize) {
                let ($($ptr,)+) = ptrs;

                $({
                    for i in 0..len {
                        unsafe {
                            $ptr.as_ptr().add(i).drop_in_place();
                        }
                    }
                })+
            }

            unsafe fn dealloc_ptrs(ptrs: Self::Pointers, cap: usize) {
                let ($($ptr,)+) = ptrs;

                $({
                    let layout = Layout::array::<$t>(cap).expect("invalid layout");
                    unsafe {
                        dealloc($ptr.as_ptr().cast::<u8>(), layout);
                    }
                })+
            }
        }
    };
}

macro_rules! impl_tuples_rec {
    (
        [],
        [
            $(($acc_t:ident, $acc_ptr:ident, $acc_item:ident),)*
        ],
    ) => {};
    (
        [
            ($first_t:ident, $first_ptr:ident, $first_item:ident),
            $(($t:ident, $ptr:ident, $item:ident),)*
        ],
        [
            $(($acc_t:ident, $acc_ptr:ident, $acc_item:ident),)*
        ],
    ) => {
        impl_tuple! {
            $(($acc_t, $acc_ptr, $acc_item),)*
            ($first_t, $first_ptr, $first_item),
        }

        impl_tuples_rec! {
            [
                $(($t, $ptr, $item),)*
            ],
            [
                $(($acc_t, $acc_ptr, $acc_item),)*
                ($first_t, $first_ptr, $first_item),
            ],
        }
    };
}

macro_rules! impl_tuples {
    ($(($t:ident, $ptr:ident, $item:ident),)+) => {
        impl_tuples_rec! {
            [
                $(($t, $ptr, $item),)*
            ],
            [],
        }
    };
}

impl_tuples! {
    (T1, ptr1, item1),
    (T2, ptr2, item2),
    (T3, ptr3, item3),
    (T4, ptr4, item4),
    (T5, ptr5, item5),
    (T6, ptr6, item6),
    (T7, ptr7, item7),
    (T8, ptr8, item8),
}

pub struct MultiVec<T: Tuple> {
    ptrs: T::Pointers,
    len: usize,
    cap: usize,
}

impl<T: Tuple> MultiVec<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            ptrs: T::DANGLING_ITEMS,
            len: 0,
            cap: 0,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        unsafe { T::ptrs_as_slices(self.ptrs, self.len) }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::MutSlices<'_> {
        unsafe { T::ptrs_as_mut_slices(self.ptrs, self.len) }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T::Ref<'_>> {
        T::get(self.as_slices(), index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<T::RefMut<'_>> {
        T::get_mut(self.as_mut_slices(), index)
    }

    pub fn push(&mut self, items: T) {
        if self.len == self.cap {
            let new_cap = self.cap.checked_mul(2).expect("capacity overflow").max(4);
            unsafe {
                T::realloc_ptrs(&mut self.ptrs, self.cap, new_cap);
            }
            self.cap = new_cap;
        }

        unsafe {
            T::write_items(self.ptrs, items, self.len);
        }
        self.len += 1;
    }

    pub fn clear(&mut self) {
        unsafe {
            T::drop_items(self.ptrs, self.len);
        }
        self.len = 0;
    }
}

impl<T: Tuple> Drop for MultiVec<T> {
    fn drop(&mut self) {
        if self.cap > 0 {
            unsafe {
                T::drop_items(self.ptrs, self.len);
                T::dealloc_ptrs(self.ptrs, self.cap);
            }
        }
    }
}

impl<T: Tuple> Default for MultiVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Tuple> fmt::Debug for MultiVec<T>
where
    for<'a> T::Ref<'a>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        let mut list = f.debug_list();
        for i in 0..self.len() {
            let items = T::get(slices, i).unwrap();
            list.entry(&items);
        }
        list.finish()
    }
}
