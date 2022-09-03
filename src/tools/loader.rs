use { 
    std::{
        collections::HashMap, 
        sync::Arc
    },
    anyhow::{Error},
};

///
/// trait who represent a loadable object.
/// Object must be sized.
/// 
pub trait Loadable 
where 
    Self: Sized {
    fn load(path: &String) -> Result<Self, Error>;
}

///
/// Struct to load generic object from a file.
/// Save the object in a hashmap to only load it once.
/// The object need to implement Loadable trait.
/// 
pub struct Loader<T> 
where 
    T : Loadable,
{
    item_loaded: HashMap<String, Arc<T>>,
}

impl<T> Loader<T>
where 
    T : Loadable,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn load(&mut self, path: &String) -> Result<Arc<T>, Error> {
        if let Some(item) = self.item_loaded.get(path) {
            Ok(item.clone())   //case where item is already loaded
        }
        else {      //case where we need to load a new item            
            let item = Arc::new(T::load(path)?);
            self.item_loaded.insert(path.clone(), item.clone());
            Ok(item.clone())
        }
    }
}

impl<T> Default for Loader<T>
where 
    T : Loadable
{
    fn default() -> Self {
        Self {
            item_loaded: HashMap::default(),
        }
    }
}