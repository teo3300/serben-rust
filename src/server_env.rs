use std::collections::HashMap;

pub struct Env {
    mime_map: HashMap<String, String>,
}

macro_rules! mime_map_init_macro{
    ($($key:expr => $val:expr),*) => {
        {
            let mut map = HashMap::new();
            $(
                map.insert($key.to_string(), $val.to_string());
            )*
            map
        }
    }
}

impl Env {
    pub fn new() -> Self {
        Self { mime_map: mime_map_init_macro!(
            "js" => "javascript"
        ) }
    }

    pub fn get_mime(&self, ext: &str) -> String {
        match self.mime_map.get(ext) {
            Some(val) => val,
            None => ext
        }.to_string()
    }
}
