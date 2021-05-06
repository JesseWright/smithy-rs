/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use smithy_xml::decode::{expect_data, Document, ScopedDecoder, XmlError};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Debug)]
enum FooEnum {
    Unknown(String),
}

impl<'a> From<&'a str> for FooEnum {
    fn from(s: &'a str) -> Self {
        Self::Unknown(s.to_string())
    }
}

#[derive(Eq, PartialEq, Debug)]
struct FlatXmlMap {
    my_map: HashMap<String, FooEnum>,
}

#[derive(Eq, PartialEq, Debug)]
struct XmlMap {
    values: HashMap<String, FooEnum>,
}

#[derive(Eq, PartialEq, Debug)]
struct XmlAttribute {
    foo: String,
    bar: String,
}

fn deserialize_xml_attribute(inp: &str) -> Result<XmlAttribute, XmlError> {
    let mut doc = Document::new(inp);
    let mut root = doc.root_element()?;
    #[allow(unused_assignments)]
    let mut foo: Option<String> = None;
    let mut bar: Option<String> = None;
    foo = root.start_el().attr("foo").map(|attr| attr.to_string());
    while let Some(mut tag) = root.next_tag() {
        if tag.start_el().matches("bar") {
            bar = Some(expect_data(&mut tag)?.to_string());
        }
    }
    Ok(XmlAttribute {
        foo: foo.ok_or(XmlError::Other { msg: "missing foo" })?,
        bar: bar.ok_or(XmlError::Other { msg: "missing bar" })?,
    })
}

fn deserialize_flat_xml_map(inp: &str) -> Result<FlatXmlMap, XmlError> {
    let mut doc = Document::new(inp);
    let mut root = doc.root_element()?;
    let mut my_map: Option<HashMap<String, FooEnum>> = None;
    while let Some(mut tag) = root.next_tag() {
        if tag.start_el().matches("myMap") {
            let mut _my_map = my_map.unwrap_or_default();
            deserialize_foo_enum_map_entry(&mut tag, &mut _my_map)?;
            my_map = Some(_my_map);
        }
    }
    Ok(FlatXmlMap {
        my_map: my_map.unwrap(),
    })
}

fn deserialize_xml_map(inp: &str) -> Result<XmlMap, XmlError> {
    let mut doc = Document::new(inp);
    let mut root = doc.root_element()?;
    let mut my_map: Option<HashMap<String, FooEnum>> = None;
    while let Some(mut tag) = root.next_tag() {
        if tag.start_el().matches("values") {
            my_map = Some(deserialize_foo_enum_map(&mut tag)?);
        }
    }
    Ok(XmlMap {
        values: my_map.ok_or(XmlError::Other { msg: "missing map" })?,
    })
}

fn deserialize_foo_enum_map(
    decoder: &mut ScopedDecoder,
) -> Result<HashMap<String, FooEnum>, XmlError> {
    let mut out: HashMap<String, FooEnum> = HashMap::new();
    while let Some(mut tag) = decoder.next_tag() {
        if tag.start_el().matches("entry") {
            deserialize_foo_enum_map_entry(&mut tag, &mut out)?;
        }
    }
    Ok(out)
}

fn deserialize_foo_enum_map_entry(
    decoder: &mut ScopedDecoder,
    out: &mut HashMap<String, FooEnum>,
) -> Result<(), XmlError> {
    let mut k: Option<String> = None;
    let mut v: Option<FooEnum> = None;
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("key") => k = Some(expect_data(&mut tag)?.to_string()),
            s if s.matches("value") => v = Some(FooEnum::from(expect_data(&mut tag)?)),
            _ => {}
        }
    }
    match (k, v) {
        (Some(k), Some(v)) => {
            out.insert(k, v);
        }
        _ => {
            return Err(XmlError::Other {
                msg: "missing key value in map",
            })
        }
    }
    Ok(())
}

#[test]
fn deserialize_map_test() {
    let xml = r#"<Foo>
    <values>
        <entry>
            <key>example-key1</key>
            <ignore><this><key>hello</key></this></ignore>
            <value>example1</value>
        </entry>
        <entry>
            <key>example-key2</key>
            <value>example2</value>
        </entry>
    </values>
</Foo>"#;

    let mut out = HashMap::new();
    out.insert("example-key1".to_string(), FooEnum::from("example1"));
    out.insert("example-key2".to_string(), FooEnum::from("example2"));
    assert_eq!(
        deserialize_xml_map(xml).expect("valid"),
        XmlMap { values: out }
    )
}

pub fn deserialize_nested_string_list(
    decoder: &mut ScopedDecoder,
) -> Result<std::vec::Vec<std::vec::Vec<std::string::String>>, XmlError> {
    let mut out = std::vec::Vec::new();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("member") => {
                out.push(deserialize_string_list(&mut tag)?);
            }
            _ => {}
        }
    }
    Ok(out)
}

pub fn deserialize_string_list(
    decoder: &mut ScopedDecoder,
) -> Result<std::vec::Vec<std::string::String>, XmlError> {
    let mut out = std::vec::Vec::new();
    while let Some(mut tag) = decoder.next_tag() {
        match dbg!(tag.start_el()) {
            s if s.matches("member") => {
                out.push(dbg!({
                    smithy_xml::decode::expect_data(&mut tag)?.to_string()
                }));
            }
            _ => {}
        };
    }
    println!("done");
    Ok(out)
}

#[test]
fn test_nested_string_list() {
    let xml = r#"
            <nestedStringList>
                <member>
                    <member>foo</member>
                    <member>bar</member>
                </member>
                <member>
                    <member>baz</member>
                    <member>qux</member>
                </member>
            </nestedStringList>
   "#;
    let mut doc = Document::new(xml);
    let mut root = doc.root_element().unwrap();
    assert_eq!(
        deserialize_nested_string_list(&mut root).unwrap(),
        vec![vec!["foo", "bar"], vec!["baz", "qux"]]
    );
}

#[test]
fn deserialize_flat_map_test() {
    let xml = r#"<FlattenedXmlMapInputOutput>
			    <myMap>
			        <key>foo</key>
			        <value>Foo</value>
			    </myMap>
			    <myMap>
			        <key>baz</key>
			        <value>Baz</value>
			    </myMap>
			</FlattenedXmlMapInputOutput>"#;

    let mut out = HashMap::new();
    out.insert("foo".to_string(), FooEnum::from("Foo"));
    out.insert("baz".to_string(), FooEnum::from("Baz"));
    assert_eq!(
        deserialize_flat_xml_map(xml),
        Ok(FlatXmlMap { my_map: out })
    )
}

#[test]
fn test_deserialize_xml_attribute() {
    let xml = r#"<MyStructure foo="example">
    <bar>examplebar</bar>
</MyStructure>"#;
    assert_eq!(
        deserialize_xml_attribute(xml).expect("valid"),
        XmlAttribute {
            foo: "example".to_string(),
            bar: "examplebar".to_string()
        }
    );
}
