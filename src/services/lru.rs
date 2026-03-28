use hashbrown::HashMap;
use std::hash::Hash;

pub struct TimestampLruCache<K, V> {
    map: HashMap<K, (V, u64)>,
    capacity: usize,
    access_counter: u64,
}

impl<K, V> TimestampLruCache<K, V>
where 
    K: Clone + Eq + Hash,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            capacity,
            access_counter: 0,
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.access_counter += 1;
        if let Some((value, timestamp)) = self.map.get_mut(key) {
            *timestamp = self.access_counter;
            Some(value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.access_counter += 1;
        if let Some((value, timestamp)) = self.map.get_mut(key) {
            *timestamp = self.access_counter;
            Some(value)
        } else {
            None
        }
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.access_counter += 1;
        
        // Evict least recently used if at capacity
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            let lru_key = self.map
                .iter()
                .min_by_key(|(_, (_, timestamp))| *timestamp)
                .map(|(k, _)| k.clone());
            
            if let Some(key_to_evict) = lru_key {
                self.map.remove(&key_to_evict);
            }
        }
        
        self.map
            .insert(key, (value, self.access_counter))
            .map(|(old_value, _)| old_value)
    }
    
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
}