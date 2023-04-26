
// Utility type for matching partial paths against HTTP request URI paths,
// this code is meant only to cover the very specific needs of HttpServerPlugin

#[derive(Clone, Default, PartialEq)]
pub struct HttpPath {
    parts: Vec<String>,
}


impl HttpPath {

    pub fn new() -> Self {
        return HttpPath::default();
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        if self.parts.len() > 0 {
            if self.parts.len() == 1 {
                result.push_str("/");
            } else {
                result.push_str(self.parts.join("/").as_str());
            }
        }
        return result;
    }

    pub fn push(&mut self, str: &str) {
        if self.parts.len() == 0 { self.parts.push(String::new()); }
        self.parts.push(String::from(str));
    }

    pub fn starts_with(&self, other: &Self) -> bool {
        if self.parts.len() < other.parts.len() { return false; }
        for (i, part) in other.parts.iter().enumerate() {
            if self.parts[i].ne(part) { return false; }
        }
        return true;
    }

}


impl From<&str> for HttpPath {

    fn from(str: &str) -> Self {
        let mut path = HttpPath::new();
        if str == "" { 
            return path; 
        }
        if str == "/" {
            path.parts.push(String::new());
            return path;
        }
        path.parts = str.split("/").map(|str| String::from(str)).collect();
        return path;
    }

}


impl std::fmt::Display for HttpPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


impl std::fmt::Debug for HttpPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.parts)
    }
}


#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _ = HttpPath::new();
    }

    #[test]
    fn new_is_empty() {
        let path = HttpPath::new();
        let facit = Vec::<String>::new();
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn default() {
        let _ = HttpPath::default();
    }

    #[test]
    fn default_is_empty() {
        let path = HttpPath::default();
        let facit = Vec::<String>::new();
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn from_emptystring() {
        let path = HttpPath::from("");
        let facit = Vec::<String>::new();
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn from_root() {
        let path = HttpPath::from("/");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn from_foo() {
        let path = HttpPath::from("/foo");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        facit.push(String::from("foo"));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn from_foo_bar() {
        let path = HttpPath::from("/foo/bar");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        facit.push(String::from("foo"));
        facit.push(String::from("bar"));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn from_foo_bar_baz() {
        let path = HttpPath::from("/foo/bar/baz");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        facit.push(String::from("foo"));
        facit.push(String::from("bar"));
        facit.push(String::from("baz"));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn push_on_empty() {
        let mut path = HttpPath::from("");
        path.push("foo");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        facit.push(String::from("foo"));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn push_on_root() {
        let mut path = HttpPath::from("/");
        path.push("foo");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        facit.push(String::from("foo"));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn push_on_foo() {
        let mut path = HttpPath::from("/foo");
        path.push("bar");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        facit.push(String::from("foo"));
        facit.push(String::from("bar"));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn push_on_foo_bar() {
        let mut path = HttpPath::from("/foo/bar");
        path.push("baz");
        let mut facit = Vec::<String>::new();
        facit.push(String::from(""));
        facit.push(String::from("foo"));
        facit.push(String::from("bar"));
        facit.push(String::from("baz"));
        assert_eq!(format!("{:?}",path), format!("{:?}",facit))
    }

    #[test]
    fn empty_to_string() {
        let path = HttpPath::from("");
        let facit = String::from("");
        assert_eq!(format!("{}",path.to_string()), format!("{}",facit))
    }

    #[test]
    fn root_to_string() {
        let path = HttpPath::from("/");
        let facit = String::from("/");
        assert_eq!(format!("{}",path.to_string()), format!("{}",facit))
    }

    #[test]
    fn foo_to_string() {
        let path = HttpPath::from("/foo");
        let facit = String::from("/foo");
        assert_eq!(format!("{}",path.to_string()), format!("{}",facit))
    }

    #[test]
    fn foo_bar_to_string() {
        let path = HttpPath::from("/foo/bar");
        let facit = String::from("/foo/bar");
        assert_eq!(format!("{}",path.to_string()), format!("{}",facit))
    }

    #[test]
    fn foo_bar_baz_to_string() {
        let path = HttpPath::from("/foo/bar/baz");
        let facit = String::from("/foo/bar/baz");
        assert_eq!(format!("{}",path.to_string()), format!("{}",facit))
    }

    #[test]
    fn empty_starts_with_empty() {
        let a = HttpPath::from("");
        let b = HttpPath::from("");
        assert_eq!(a.starts_with(&b), true)
    }

    #[test]
    fn root_starts_with_root() {
        let a = HttpPath::from("/");
        let b = HttpPath::from("/");
        assert_eq!(a.starts_with(&b), true)
    }

    #[test]
    fn foo_starts_with_foo() {
        let a = HttpPath::from("/foo");
        let b = HttpPath::from("/foo");
        assert_eq!(a.starts_with(&b), true)
    }

    #[test]
    fn foobar_starts_with_foobar() {
        let a = HttpPath::from("/foo/bar");
        let b = HttpPath::from("/foo/bar");
        assert_eq!(a.starts_with(&b), true)
    }

    #[test]
    fn foobar_starts_with_empty() {
        let a = HttpPath::from("/foo/bar");
        let b = HttpPath::from("");
        assert_eq!(a.starts_with(&b), true)
    }

    #[test]
    fn foobar_starts_with_root() {
        let a = HttpPath::from("/foo/bar");
        let b = HttpPath::from("/");
        assert_eq!(a.starts_with(&b), true)
    }

    #[test]
    fn foobar_starts_with_foo() {
        let a = HttpPath::from("/foo/bar");
        let b = HttpPath::from("/foo");
        assert_eq!(a.starts_with(&b), true)
    }

    #[test]
    fn empty_starts_with_root() {
        let a = HttpPath::from("");
        let b = HttpPath::from("/");
        assert_eq!(a.starts_with(&b), false)
    }

    #[test]
    fn empty_starts_with_foo() {
        let a = HttpPath::from("");
        let b = HttpPath::from("/foo");
        assert_eq!(a.starts_with(&b), false)
    }

    #[test]
    fn empty_starts_with_foobar() {
        let a = HttpPath::from("");
        let b = HttpPath::from("/foo/bar");
        assert_eq!(a.starts_with(&b), false)
    }

    #[test]
    fn root_starts_with_foo() {
        let a = HttpPath::from("/");
        let b = HttpPath::from("/foo");
        assert_eq!(a.starts_with(&b), false)
    }

    #[test]
    fn root_starts_with_foobar() {
        let a = HttpPath::from("/");
        let b = HttpPath::from("/foo/bar");
        assert_eq!(a.starts_with(&b), false)
    }

    #[test]
    fn foo_starts_with_foobar() {
        let a = HttpPath::from("/foo");
        let b = HttpPath::from("/foo/bar");
        assert_eq!(a.starts_with(&b), false)
    }

    #[test]
    fn foobar_starts_with_foobaz() {
        let a = HttpPath::from("/foo/bar");
        let b = HttpPath::from("/foo/baz");
        assert_eq!(a.starts_with(&b), false)
    }

    #[test]
    fn barfoo_starts_with_bazfoo() {
        let a = HttpPath::from("/bar/foo");
        let b = HttpPath::from("/baz/foo");
        assert_eq!(a.starts_with(&b), false)
    }


}
