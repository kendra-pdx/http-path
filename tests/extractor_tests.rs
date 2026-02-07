use http_path::prelude::*;

#[test]
fn extract() {
    let route = extractor!("a/1:u32/c:str/{u32}/{str}");
    println!("route: {route:?}");
    if let Some(hlist_pat![_, _, _, n, s]) = route.extract(&["a", "1", "c", "2", "d"]) {
        assert_eq!(n, 2);
        assert_eq!(s, "d");
    } else {
        panic!("route did not match")
    }
}
