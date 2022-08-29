use std::path::Path;

struct PlutoPath {
    str_repr: String,
}

impl PlutoPath {
    pub fn new(path: &str) -> Self {
        for _i in path.chars() {}

        todo!()
    }
}

impl AsRef<Path> for PlutoPath {
    fn as_ref(&self) -> &Path {
        todo!()
    }
}
