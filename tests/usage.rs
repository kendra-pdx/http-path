use http_path::prelude::*;

#[test]
fn usage() {
    let route = extractor!("a/1:u32/c:str/{u32}/{str}");
    println!("route: {route:?}");

    let path = Path::parse("/a/1/c/2/d?foo=bar").unwrap();
    println!("path: {path:?}");

    let matched = route.extract(path.segments());
    println!("matched: {matched:?}");

    if let Some(hlist_pat![_, _, _, n, s]) = matched {
        assert_eq!(n, 2);
        assert_eq!(s, "d");
    } else {
        panic!("route did not match")
    }
}
