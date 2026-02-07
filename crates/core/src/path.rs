use alloc::vec::Vec;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{self:?}")]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    PathParseError,
    NotImplemented,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Path<'p> {
    path: Vec<&'p str>,
    query: Option<Query<'p>>,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Query<'p> {
    query: &'p str,
}

impl<'p> Path<'p> {
    pub fn parse(path: &'p str) -> Result<Self, Error> {
        // Split the path and query string at '?'
        let (path_part, query_part) = match path.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (path, None),
        };

        // Parse the path segments
        let path_segments: Vec<&'p str> = path_part.split('/').filter(|s| !s.is_empty()).collect();

        // Create the query if present
        let query = query_part.map(|q| Query { query: q });

        Ok(Path {
            path: path_segments,
            query,
        })
    }

    pub fn segments(&self) -> &[&'p str] {
        &self.path
    }

    pub fn query(&self) -> Option<&Query<'p>> {
        self.query.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QueryKV<'p> {
    Key(&'p str),
    KeyValue(&'p str, &'p str),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum QueryKeyResult<'p> {
    None,
    Single(QueryKV<'p>),
    Multiple(Vec<QueryKV<'p>>),
}

impl<'p> Query<'p> {
    pub fn get(&self, key: &str) -> QueryKeyResult<'p> {
        let mut results = Vec::new();
        for pair in self.query.split('&') {
            let mut iter = pair.splitn(2, '=');
            let k = iter.next().unwrap_or("");
            let v = iter.next();

            if k == key {
                if let Some(value) = v {
                    results.push(QueryKV::KeyValue(k, value));
                } else {
                    results.push(QueryKV::Key(k));
                }
            }
        }

        match results.len() {
            0 => QueryKeyResult::None,
            1 => QueryKeyResult::Single(results[0]),
            _ => QueryKeyResult::Multiple(results),
        }
    }
}

impl<'p> IntoIterator for QueryKeyResult<'p> {
    type Item = QueryKV<'p>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            QueryKeyResult::None => Vec::new().into_iter(),
            QueryKeyResult::Single(kv) => alloc::vec![kv].into_iter(),
            QueryKeyResult::Multiple(kvs) => kvs.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("/users/123/profile?active=true&sort=asc", &["users", "123", "profile"], Some("active=true&sort=asc"))]
    #[case("/api/v1/users", &["api", "v1", "users"], None)]
    #[case("/", &[], None)]
    #[case("", &[], None)]
    #[case("/path", &["path"], None)]
    #[case("/path/", &["path"], None)]
    #[case("//path//to///resource", &["path", "to", "resource"], None)]
    #[case("/?query=value", &[], Some("query=value"))]
    #[case("/path?", &["path"], Some(""))]
    #[case("/path?key=value&other=data", &["path"], Some("key=value&other=data"))]
    #[case("/a/b/c?x=1&y=2&z=3", &["a", "b", "c"], Some("x=1&y=2&z=3"))]
    #[case("/path?query?with?question?marks", &["path"], Some("query?with?question?marks"))]
    fn parse_path(
        #[case] input: &str,
        #[case] expected_path: &[&str],
        #[case] expected_query: Option<&str>,
    ) {
        let parsed = Path::parse(input).unwrap();

        assert_eq!(parsed.segments(), expected_path);
        match expected_query {
            Some(q) => {
                assert!(parsed.query().is_some());
                assert_eq!(parsed.query().unwrap().query, q);
            }
            None => assert!(parsed.query().is_none()),
        }
    }

    #[rstest]
    #[case(
        "key=value",
        "key",
        QueryKeyResult::Single(QueryKV::KeyValue("key", "value"))
    )]
    #[case("key=value", "other", QueryKeyResult::None)]
    #[case("flag", "flag", QueryKeyResult::Single(QueryKV::Key("flag")))]
    #[case("flag", "other", QueryKeyResult::None)]
    #[case(
        "key=value&other=data",
        "key",
        QueryKeyResult::Single(QueryKV::KeyValue("key", "value"))
    )]
    #[case(
        "key=value&other=data",
        "other",
        QueryKeyResult::Single(QueryKV::KeyValue("other", "data"))
    )]
    #[case("key=value&key=another", "key", QueryKeyResult::Multiple(alloc::vec![QueryKV::KeyValue("key", "value"), QueryKV::KeyValue("key", "another")]))]
    #[case("flag&flag", "flag", QueryKeyResult::Multiple(alloc::vec![QueryKV::Key("flag"), QueryKV::Key("flag")]))]
    #[case(
        "key=&other=value",
        "key",
        QueryKeyResult::Single(QueryKV::KeyValue("key", ""))
    )]
    #[case("a=1&b=2&c=3&a=4", "a", QueryKeyResult::Multiple(alloc::vec![QueryKV::KeyValue("a", "1"), QueryKV::KeyValue("a", "4")]))]
    #[case(
        "a=1&b=2&c=3",
        "b",
        QueryKeyResult::Single(QueryKV::KeyValue("b", "2"))
    )]
    #[case("prefix=value", "pre", QueryKeyResult::None)]
    #[case(
        "key=value=with=equals",
        "key",
        QueryKeyResult::Single(QueryKV::KeyValue("key", "value=with=equals"))
    )]
    #[case("", "key", QueryKeyResult::None)]
    #[case("key", "key", QueryKeyResult::Single(QueryKV::Key("key")))]
    fn query_get(#[case] query_string: &str, #[case] key: &str, #[case] expected: QueryKeyResult) {
        let query = Query {
            query: query_string,
        };
        let result = query.get(key);
        assert_eq!(result, expected);
    }
}
