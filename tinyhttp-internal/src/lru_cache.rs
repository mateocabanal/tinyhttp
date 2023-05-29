use std::{
    fmt::Debug,
};


#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct LRUCache<T> {
    pub(crate) array: Box<[T]>,
}

pub(crate) struct Iter<'a, T> {
    cache: &'a mut LRUCache<T>,
    pos: usize,
}

#[allow(dead_code)]
impl<T: Debug + Clone> LRUCache<T> {
    //    pub(crate) fn new() -> Self {
    //        //let array = unsafe { Box::<[(i32, T)]>::new_zeroed_slice(capacity as usize).assume_init() };
    //        let array = Vec::<T>::with_capacity(10).into_boxed_slice();
    //        LRUCache { array }
    //    }

    //    fn get(&mut self, key: i32) -> Option<T> {
    //        //        let key = key as u16;
    //        if let Some(s) = self.array.iter().position(|p| p.0 == key) {
    //            self.array.make_contiguous()[0..=s].rotate_right(1);
    //            Some(self.array[0].1.clone)
    //        } else {
    //            None
    //        }
    //    }

    pub(crate) fn get(&mut self, idx: usize) -> Option<&T> {
        self.touch(idx)?;

        self.array.get(0)
    }

    //    pub(crate) fn push(&mut self, value: T) {
    //        self.arraypush_front(value);
    //    }

    pub(crate) fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    pub(crate) fn touch(&mut self, idx: usize) -> Option<()> {

        #[cfg(debug_assertions)]
        let clone = self.array.get(idx).cloned();
        log::debug!("Moved element {}\n {:#?} to front", idx, clone?);

        self.array.get_mut(0..=idx)?.rotate_right(1);

        //log::debug!("LRU: {:#?}", self.array.get(0).unwrap());

        Some(())
    }

    pub(crate) fn iter<'a>(&'a mut self) -> Iter<'a, T> {
        Iter {
            cache: self,
            pos: 0,
        }
    }
}

//impl<T> IntoIterator for LRUCache<T> {
//    type Item = T;
//    type IntoIter = IntoIter<Self::Item>;
//
//    fn into_iter(self) -> Self::IntoIter {
//        self.array.into_iter()
//    }
//}

impl<'a, T: Debug + Clone> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.cache.get(self.pos)?;
        //log::debug!("GOT: {:#?}", entry);
        self.pos += 1;
        Some(entry).cloned()
    }
}

mod tests {
    
    use super::LRUCache;

    #[test]
    fn lru_test_1() -> Result<(), Box<dyn std::error::Error>> {
        let mut test = Vec::new();
        for i in 0..5 {
            test.push(i);
        }

        let mut lru = LRUCache {
            array: test.into_boxed_slice(),
        };

        for i in lru.iter() {
            if i == 3 {
                break;
            }
        }

        assert_eq!(lru.array[0], 3);
        assert_eq!(lru.array[1], 2);
        assert_eq!(lru.array[2], 1);
        assert_eq!(lru.array[3], 0);
        assert_eq!(lru.array[4], 4);
        
        let val = lru.iter().find(|i| *i == 4).unwrap();

        assert_eq!(val, lru.array[0]);

        Ok(())
    }
}
