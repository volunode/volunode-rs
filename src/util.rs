extern crate std;
extern crate treexml;

use std::fmt::Debug;
use std::str::FromStr;

use errors;

pub fn parse_node(s: &str) -> Result<treexml::Element, errors::Error> {
    let doc = try!(treexml::Document::parse(s.as_bytes()));

    Ok(try!(doc.root.ok_or(
        errors::Error::NullError("Root is empty".into()),
    )))
}

pub fn eval_node_contents<T>(node: &treexml::Element) -> Option<T>
where
    T: FromStr,
{
    match node.text {
        Some(ref v) => v.parse::<T>().ok(),
        _ => None,
    }
}

pub fn any_text(node: &treexml::Element) -> Option<String> {
    if node.cdata.is_some() {
        return node.cdata.clone();
    }
    if node.text.is_some() {
        return node.text.clone();
    }
    return None;
}

pub fn trimmed_optional(e: &Option<String>) -> Option<String> {
    e.clone().map(|v| v.trim().into())
}

pub fn deserialize_failed(n: &treexml::Element) -> errors::Error {
    errors::Error::DataParseError(format!("Failed to deserialize node: {:?}", n))
}

pub fn deserialize_node<T>(
    name: &str,
    n: &treexml::Element,
    result: &mut T,
) -> Result<(), errors::Error>
where
    T: std::str::FromStr,
{
    if &n.name != name {
        Ok(())
    } else {
        match eval_node_contents::<T>(n) {
            Some(mut v) => {
                std::mem::swap(result, &mut v);
                Ok(())
            }
            None => Err(deserialize_failed(n)),
        }
    }
}

pub fn serialize_node<T>(name: &str, v: &T) -> treexml::ElementBuilder where T: std::fmt::Display {
    let mut e = treexml::ElementBuilder::new(name);
    e.text(format!("{}", &v));

    e
}
