// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth
use nom::{
    IResult,
    sequence::{tuple, pair},
    branch::{alt},
    bytes::complete::{tag, take_while, take, take_until, take_while1},
    character::complete::{multispace0, multispace1, char, digit1, hex_digit1},
    multi::{fold_many0, fold_many1, many0}
};
use super::util::option;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

use super::ast;

/// 'package' name=FQN
fn parse_package(input: &str) -> IResult<&str, String> {
    match tuple((multispace0, tag("package"), multispace0, parse_fqn))(input) {
        Ok((rem, (_ws1, _tag, _ws2, fqn))) => Ok((rem, fqn)),
        Err(err) => Err(err),
    }
}

/// Parse string for a FRANCA identifier which is an XTEXT ID token.
/// XTEXT:
///  terminal ID: ('^')?('a'..'z'|'A'..'Z'|'_') ('a'..'z'|'A'..'Z'|'_'|'0'..'9')*;
fn parse_identifier(input: &str) -> IResult<&str, &str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\^)?[a-zA-Z_][a-zA-Z0-9_]*").unwrap();
    }
    match RE.find(input) {
        None => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::RegexpMatch))),
        Some(reg_match) => {
            Ok((&input[reg_match.end()..], &input[reg_match.start()..reg_match.end()]))
        }
    }
}

fn quoted_string(c: char) -> impl Fn(&str) -> nom::IResult<&str, String> {
    move |i: &str| {
        let (r, v) = nom::sequence::tuple(
            ( char(c), take_while(|ch: char| { ch != c})))(i)?;
        let (r1, _) = take(1usize)(r)?;
        Ok((r1, v.1.to_string()))
    }
}

fn parse_string(input: &str) -> IResult<&str, String> {
    nom::branch::alt((quoted_string('"'), quoted_string('\'') ))(input)
}

/// Parses a FQN (Fully Qualified Name).
/// FQN: ID ('.' ID)*;
fn parse_fqn(input: &str) -> IResult<&str, String> {
    match nom::sequence::pair(parse_identifier,
                              many0(pair(tag("."), parse_identifier))
    )(input) {
        Err(err) => Err(err),
        Ok((rem, fqn)) => {
            let mut fqn_vec = vec![fqn.0];
            for comp in fqn.1 {
                fqn_vec.push(comp.1);
            }
            Ok((rem, fqn_vec.join(".")))
        }
    }
}

fn parse_import_model(input: &str) -> IResult<&str, ast::Import> {
    let (r, v) = tuple(
        (multispace0, tag("import"), multispace1, tag("model"),
         multispace1, parse_string,
        ))(input)?;
    Ok((r, ast::Import{ uri: v.5.to_string(), namespace: "".to_string()}))
}

fn parse_imported_fqn(input: &str) -> IResult<&str, String> {
    let (r, v) = pair(parse_fqn, option( tag(".*")))(input)?;
    if v.1.is_none() {
        Ok((r, v.0.to_string()))
    }
    else {
        Ok((r, format!("{}.*", v.0)))
    }
}

fn parse_import_from(input: &str) -> IResult<&str, ast::Import> {
    let (r, v) = tuple((
        multispace0, tag("import"), multispace1, parse_imported_fqn, multispace1,
        tag("from"), multispace1, parse_string,
    ))(input)?;
    Ok((r, ast::Import{ uri: v.7.to_string(), namespace: v.3}))
}

/// Import : 'import' (importedNamespace=ImportedFQN 'from' | 'model') importURI=STRING;
fn parse_import(input: &str) -> IResult<&str, ast::Import> {
    alt((parse_import_from, parse_import_model))(input)
}

/// FAnnotationBlock returns FAnnotationBlock:
/// 	'<**' (elements+=FAnnotation)+ '**>';
fn parse_annotation(input: &str) -> IResult<&str, Option<String>> {
    match nom::sequence::tuple((tag("<**"), take_until("**>"), tag("**>"), multispace0,
    ))(input) as IResult<&str, (&str, &str, &str, &str)> {
        Ok((r, v))  => Ok((r, Some(v.1.to_string()))),
        Err(_)                => Ok((input, None))
    }
}

/// 'version' FVersion returns FVersion:
/// 	{FVersion}
/// 	'{'
/// 		'major' major=INT
/// 		'minor' minor=INT
///     '}';
fn parse_version(input: &str) -> IResult<&str, Option<(u32, u32)>> {
    match nom::sequence::tuple((
        tag("version"), multispace0, tag("{"), multispace0, tag("major"), multispace1, digit1,
        multispace1, tag("minor"), multispace1, digit1, multispace0, tag("}"), multispace0
    ))(input) as IResult<&str, (&str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str)>  {
        Ok((r, v)) => Ok((r, Some(( v.6.parse::<u32>().unwrap(), v.10.parse::<u32>().unwrap())))),
        Err(_) => Ok((input, None))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum ModuleContent {
    Interface(ast::Interface),
    TypeCollection(ast::TypeCollection)
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum InterfaceContent {
    Attribute(ast::Attribute),
    Method(),
    Broadcast(),
    Constant(),
    Type(),
}

fn parse_type_ref(input: &str) -> IResult<&str, ast::TypeRef> {
    // todo IntegerInterval not recognized
    let (r, v) = parse_fqn(input)?;
    let tr = match v.as_str() {
        "undefined" => ast::TypeRef::Undefined,
        "Int8" => ast::TypeRef::Int8,
        "UInt8" => ast::TypeRef::UInt8,
        "Int16" => ast::TypeRef::Int16,
        "UInt16" => ast::TypeRef::UInt16,
        "Int32" => ast::TypeRef::Int32,
        "UInt32" => ast::TypeRef::UInt32,
        "Int64" => ast::TypeRef::Int64,
        "UInt64" => ast::TypeRef::UInt64,
        "Boolean" => ast::TypeRef::Boolean,
        "String" => ast::TypeRef::String,
        "Float" => ast::TypeRef::Float,
        "Double" => ast::TypeRef::Double,
        "ByteBuffer" => ast::TypeRef::ByteBuffer,
        _ => ast::TypeRef::Derived(v),
    };
    Ok((r, tr))
}

fn parse_array_specifier(input: &str) -> IResult<&str, bool> {
    let (r, v) = option(tuple((tag("["), multispace0, tag("]"), multispace0)))
        (input)?;
    Ok((r, v.is_some()))
}

fn parse_attribute(input: &str) -> IResult<&str, InterfaceContent> {
    let (r, v) = tuple((
        multispace0,
        parse_annotation, tag("attribute"), multispace0, parse_type_ref, multispace0,
        parse_array_specifier, multispace0,
        parse_identifier, multispace0,
        fold_many0( alt((tag("readonly"), tag("noRead"), tag("noSubscription"), multispace1 )),
            || (false, false, false), |mut sp, v| {
                // println!("sp {:?} ({})", sp, v);
                match v {
                    "readonly" => sp.0 = true,
                    "noRead" => sp.1 = true,
                    "noSubscription" => sp.2 = true,
                    _ => {},
                }
                // println!(" =>sp {:?}", sp);
                sp
            } ), multispace0
    ))(input)?;
    Ok((r, InterfaceContent::Attribute(ast::Attribute {
        annotation: v.1, name: v.8.to_string(), array: v.6, type_ref: v.4,
        read_only: v.10.0, no_subscription: v.10.2, no_read: v.10.1
    })))
}

fn parse_interface(input: &str) -> IResult<&str, ModuleContent> {
    let (r, v) = nom::sequence::tuple((
        parse_annotation, tag("interface"), multispace1, parse_identifier, multispace0,
        // todo extends and manages
        tag("{"), multispace0, parse_version, multispace0,
        fold_many0( parse_attribute, || Vec::new(),
            |mut attrs, v | {
                match v {
                    InterfaceContent::Attribute(attr) => attrs.push(attr),
                    _ => {},
                }
                attrs
            }),
        multispace0, tag("}"), multispace0
    ))(input)?;
    Ok((r, ModuleContent::Interface( ast::Interface{
        annotation: v.0, name: v.3.to_string(), version: v.7, attributes: v.9
    })))
}

fn parse_type_collection(input: &str) -> IResult<&str, ModuleContent> {
    let (r, _) = nom::sequence::tuple((
        parse_annotation, tag("typeCollection"), multispace0,
        option(parse_identifier), multispace0, tag("{"), multispace0,
        tag("}")
    ))(input)?;
    Ok((r, ModuleContent::TypeCollection(
        ast:: TypeCollection{

        })))
}

/// FModel returns FModel:
/// 	{FModel}
/// 	'package' name=FQN
/// 	(imports+=Import)*
/// 	( typeCollections+=FTypeCollection | interfaces+=FInterface	)*;
pub fn parse_module(input: &str) -> IResult<&str, ast::Module> {
    let (r, v) = nom::sequence::tuple((
        multispace0, parse_package, multispace0,
        fold_many0(pair(parse_import, multispace0), || Vec::new(),
                   |mut imports: Vec<_>, item|{ imports.push(item.0); imports} ),
        multispace0,
        fold_many0(alt((parse_interface, parse_type_collection)), || (Vec::new(), Vec::new()) ,
                   | (mut intfs_vec, mut tc_vec), item| {
                       match item {
                           ModuleContent::Interface(intfs) => intfs_vec.push(intfs),
                           ModuleContent::TypeCollection(tc) => tc_vec.push(tc),
                       };
                       (intfs_vec, tc_vec)
                   }),
        multispace0
    ))(input)?;
    Ok((r, ast::Module{ package: v.1, imports: v.3, interfaces: v.5.0, type_collections: v.5.1}))
}

fn parse_field(input: &str) -> IResult<&str, ast::Field> {
    let (r, v) = tuple((
        parse_annotation, multispace0, parse_type_ref, multispace0, parse_array_specifier,
        parse_identifier, multispace0
    ))(input)?;
    Ok((r, ast::Field{
        annotation: v.0, array: v.4, name: v.5.to_string(), type_ref: v.2 }))
}

fn parse_typedef(input: &str) -> IResult<&str, ast::Type> {
    let (r, v) = tuple((
        parse_annotation, option(tag("public ")), multispace0, tag("typedef "), multispace0,
        parse_identifier, multispace0, tag("is "), multispace0, parse_type_ref, multispace0,
        parse_array_specifier, multispace0
    ))(input)?;
    Ok((r, ast::Type::TypeDef {
        annotation: v.0, public: v.1.is_some(), name: v.5.to_string(), actual_type: v.9, array: v.11
    }))
}

fn parse_array_type(input: &str) -> IResult<&str, ast::Type> {
    let (r, v) = tuple((
        parse_annotation, option(tag("public ")), multispace0, tag("array"), multispace1,
        parse_identifier, multispace0, tag("of "), multispace0, parse_type_ref, multispace0
    ))(input)?;
    Ok((r, ast::Type::Array {
        annotation: v.0, public: v.1.is_some(), name: v.5.to_string(), element_type: v.9
    }))
}

fn parse_struct_type(input: &str) -> IResult<&str, ast::Type> {
    let (r, v) = tuple((
        parse_annotation, option(tag("public ")), multispace0, tag("struct"), multispace1,
        parse_identifier, multispace0,
        option( tuple((tag("extends "), parse_fqn))), multispace0,
        option(tag("polymorphic")), multispace0,
        tag("{"), multispace0,
        fold_many0(parse_field, || Vec::new(),
            |mut vec, field | { vec.push(field); vec}),
        tag("}"), multispace0
    ))(input)?;
    let extend_fqn = if let Some(ex) = v.7 { Some(ex.1) } else { None };
    Ok((r, ast::Type::Struct {
        annotation: v.0, public: v.1.is_some(), name: v.5.to_string(), polymorphic: v.9.is_some(),
        extends: extend_fqn, fields: v.13
    }))
}

fn parse_union_type(input: &str) -> IResult<&str, ast::Type> {
    let (r, v) = tuple ((
        parse_annotation, option(tag("public ")), multispace0, tag("union"), multispace1,
        parse_identifier, multispace0,
        option(tuple((tag("extends "), parse_fqn))), multispace0,
        tag("{"), multispace0,
        fold_many0(parse_field, || Vec::new(),
                   |mut vec, field | { vec.push(field); vec}),
        tag("}"), multispace0
    ))(input)?;
    let base = if let Some(ex) = v.7 { Some(ex.1) } else { None };
    Ok((r, ast::Type::Union {
        annotation: v.0, public: v.1.is_some(), name: v.5.to_string(), base_type: base, fields: v.11
    }))
}

fn parse_map_type(input: &str) -> IResult<&str, ast::Type> {
    let (r, v) = tuple((
        parse_annotation, option(tag("public ")), multispace0, tag("map"), multispace1,
        parse_identifier, multispace0, tag("{"), multispace0, parse_type_ref, multispace0,
        tag("to"), multispace0, parse_type_ref, multispace0, tag("}"), multispace0
    ))(input)?;
    Ok((r, ast::Type::Map {
        annotation: v.0, public: v.1.is_some(), name: v.5.to_string(), key_type: v.9, value_type: v.13
    }))
}

fn parse_integer_decimal(input: &str) -> IResult<&str, u64> {
    let (r, v) = tuple( (digit1, multispace0) )(input)?;
    Ok((r, u64::from_str(v.0).unwrap()))
}

fn parse_integer_hex(input: &str) -> IResult<&str, u64> {
    let (r, v) = tuple( (alt((tag("0x"), tag("0X"))), hex_digit1, multispace0))(input) ?;
    Ok((r, u64::from_str_radix(v.1, 16).unwrap()))
}

fn parse_integer_bin(input: &str) -> IResult<&str, u64> {
    let is_bin_digit = |c: char| { c == '0' || c == '1'};
    let (r, v) = tuple((alt((tag("0b"), tag("0B"))), take_while1(is_bin_digit),
        multispace0))(input)?;
    Ok((r, u64::from_str_radix(v.1, 2).unwrap()))
}

fn parse_integer(input: &str) -> IResult<&str, u64> {
    let (r, v) = alt((parse_integer_hex, parse_integer_bin, parse_integer_decimal))(input)?;
    Ok((r, v))
}

fn parse_enumerator(input: &str) -> IResult<&str, ast::Enumerator> {
    let (r, v) = tuple((parse_annotation, multispace0, parse_identifier, multispace0,
        option( tuple(( tag("="), multispace0, parse_integer)) ), multispace0
    ))(input)?;
    let value = if let Some(val) = v.4 { Some(val.2) } else { None };
    Ok((r, ast::Enumerator{ annotation: v.0, name: v.2.to_string(), val: value }))
}

fn parse_enumeration(input: &str) -> IResult<&str, ast::Type> {
    let (r, v) = tuple((parse_annotation, option(tag("public ")),  multispace0,
        tag("enumeration"), multispace1, parse_identifier, multispace0,
        option(tuple((tag("extends "), parse_type_ref))), multispace0, tag("{"), multispace0,
        fold_many1( tuple((parse_enumerator, option(tag(","))) ) , || Vec::new(),
            |mut vec, item| {
                vec.push(item.0);
                vec
        }),
        multispace0, tag("}"), multispace0
    ))(input)?;
    let extension = if let Some(ex) = v.7 {Some(ex.1)} else {None};
    Ok((r, ast::Type::Enumeration {
        annotation: v.0, public: v.1.is_some(), name: v.5.to_string(), base_type: extension,
        enumerators: v.11
    }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_enumeration() {
        assert_eq!(parse_enumeration("public enumeration MyEnum { A=1 B=100, C D }"),
            Ok(("", ast::Type::Enumeration {annotation: None, public: true, name: "MyEnum".to_string(),
                base_type: None, enumerators: vec![
                    ast::Enumerator{ annotation: None, name: "A".to_string(), val: Some(1) },
                    ast::Enumerator{ annotation: None, name: "B".to_string(), val: Some(100) },
                    ast::Enumerator{ annotation: None, name: "C".to_string(), val: None },
                    ast::Enumerator{ annotation: None, name: "D".to_string(), val: None },
                ]
        })));
    }

    #[test]
    fn test_enumerator() {
        assert_eq!(parse_enumerator(" enum_value_1 "), Ok(("", ast::Enumerator{
            annotation: None, name: "enum_value_1".to_string(), val: None })));
        assert_eq!(parse_enumerator("<** some comment **>\n SIGNAL_UNBEFUELLT = 0x12"), Ok(("", ast::Enumerator {
            annotation: Some(" some comment ".to_string()), name: "SIGNAL_UNBEFUELLT".to_string(), val: Some(0x12u64) })));
    }

    #[test]
    fn test_integer_decimal() {
        assert_eq!(parse_integer_decimal("1234"), Ok(("", 1234u64)));
        assert_eq!(parse_integer_decimal("0"), Ok(("", 0u64)));
        assert_eq!(parse_integer_decimal("65535"), Ok(("", 0xffffu64)));
    }

    #[test]
    fn test_integer_hex() {
        assert_eq!(parse_integer_hex("0x1234"), Ok(("", 0x1234u64)));
        assert_eq!(parse_integer_hex("0X0"), Ok(("", 0u64)));
        assert_eq!(parse_integer_hex("0xffffffff"), Ok(("", 0xffffffffu64)));
    }

    #[test]
    fn test_integer_binary() {
        assert_eq!(parse_integer_bin("0b1101"), Ok(("", 13u64)));
        assert_eq!(parse_integer_bin("0B11110001"), Ok(("", 0xf1u64)));
        assert_eq!(parse_integer_bin("0b0"), Ok(("", 0)));
    }

    #[test]
    fn test_map_type() {
        assert_eq!(parse_map_type("public map myMap { type_x to Boolean }"),
            Ok(("", ast::Type::Map {
                annotation: None, public: true, name: "myMap".to_string(),
                key_type: ast::TypeRef::Derived("type_x".to_string()), value_type: ast::TypeRef::Boolean
        })));
    }

    #[test]
    fn test_union_type() {
        assert_eq!(parse_union_type("<**comment**> union A_Union extends X_Type {\n <**a**> Int32 counter \n Int64 long_counter }"),
            Ok(("", ast::Type::Union {
                annotation: Some("comment".to_string()), public: false, base_type: Some("X_Type".to_string()),
                name: "A_Union".to_string(), fields: vec![
                    ast::Field{ annotation: Some("a".to_string()), name: "counter".to_string(), array: false, type_ref: ast::TypeRef::Int32},
                    ast::Field{ annotation: None, name: "long_counter".to_string(), array: false, type_ref: ast::TypeRef::Int64},
                ]
        })));
    }

    #[test]
    fn test_struct_type() {
        assert_eq!(parse_struct_type("public struct MyStruct {\n Int8 a\n UInt32 b String[] c} XYZ"),
            Ok(("XYZ", ast::Type::Struct { annotation: None, public: true, name: "MyStruct".to_string(),
                polymorphic: false, extends: None,
                fields: vec![
                    ast::Field{ annotation: None, name: "a".to_string(), array: false, type_ref: ast::TypeRef::Int8},
                    ast::Field{ annotation: None, name: "b".to_string(), array: false, type_ref: ast::TypeRef::UInt32},
                    ast::Field{ annotation: None, name: "c".to_string(), array: true, type_ref: ast::TypeRef::String},
              ]
        })));
    }

    #[test]
    fn test_array_type() {
        assert_eq!(parse_array_type("public array MyArray of UInt32  AAA"),
                   Ok(("AAA", ast::Type::Array{annotation: None, public: true, name: "MyArray".to_string(),
                       element_type: ast::TypeRef::UInt32})));
        assert_eq!(parse_array_type("<** nothing \n here \n to see **>\n  array SomeArray of Boolean    AAA"),
                   Ok(("AAA", ast::Type::Array{annotation: Some(" nothing \n here \n to see ".to_string()),
                       public: false, name: "SomeArray".to_string(), element_type: ast::TypeRef::Boolean })));
    }

    #[test]
    fn test_typedef() {
        assert_eq!(parse_typedef("public typedef MyType is Int8  AAA"),
            Ok(("AAA", ast::Type::TypeDef{annotation: None, public: true, name: "MyType".to_string(),
                        actual_type: ast::TypeRef::Int8, array: false})));
        assert_eq!(parse_typedef("<** nothing **>\n typedef SomeArray is Boolean [  ]  AAA"),
                   Ok(("AAA", ast::Type::TypeDef{annotation: Some(" nothing ".to_string()),
                       public: false, name: "SomeArray".to_string(), actual_type: ast::TypeRef::Boolean,
                       array: true})));
    }

    #[test]
    fn test_field() {
        assert_eq!(parse_field("Boolean my_bool   "),
                   Ok(("", ast::Field{annotation: None, name: "my_bool".to_string(), array: false, type_ref: ast::TypeRef::Boolean})));
        assert_eq!(parse_field("UInt32[] an_array \nA"),
                   Ok(("A", ast::Field{annotation: None, name: "an_array".to_string(), array: true, type_ref: ast::TypeRef::UInt32})));
        assert_eq!(parse_field("<** a little comment**>\n     MyOwnType field \n"),
                   Ok(("", ast::Field{annotation: Some(" a little comment".to_string()),
                       name: "field".to_string(), array: false, type_ref: ast::TypeRef::Derived("MyOwnType".to_string())})));
    }

    #[test]
    fn test_attribute() {
        assert_eq!(parse_attribute("attribute Int8 my_int_8  }"), Ok(("}", InterfaceContent::Attribute(
            ast::Attribute{ annotation: None, name: "my_int_8".to_string(), array: false,
                read_only: false, no_subscription: false, no_read: false, type_ref: ast::TypeRef::Int8,
        }))));
        assert_eq!(parse_attribute("attribute MyType[] a readonly  noSubscription  "), Ok(("", InterfaceContent::Attribute(
            ast::Attribute{ annotation: None, name: "a".to_string(), array: true,
                read_only: true, no_subscription: true, no_read: false, type_ref: ast::TypeRef::Derived("MyType".to_string()),
        }))));
    }

    #[test]
    fn test_annotation() {
        assert_eq!(parse_annotation("<** an annotation \n comment with multiple \n lines **> adf" ),
                   Ok(("adf", Some(" an annotation \n comment with multiple \n lines ".to_string()))));
        assert_eq!(parse_annotation("<**an annotation**>\n adf" ),
                   Ok(("adf", Some("an annotation".to_string()))));
        assert_eq!(parse_annotation("\n  <*an annotation**>\n adf" ),
                   Ok(("\n  <*an annotation**>\n adf", None)));
    }

    #[test]
    fn test_version() {
        assert_eq!(parse_version("version{ major 1 minor 3}"), Ok(("", Some((1, 3)))));
        assert_eq!(parse_version("not a version"), Ok(("not a version", None)));
    }

    #[test]
    fn test_import() {
        assert_eq!(parse_import(" import a.b.c from 'a_b-file.fidl'"),
                   Ok(("", ast::Import{ uri: "a_b-file.fidl".to_string(), namespace: "a.b.c".to_string()})));
        assert_eq!(parse_import(" import model   'a_b-file.fidl' \n a new line"),
                   Ok((" \n a new line", ast::Import{ uri: "a_b-file.fidl".to_string(), namespace: String::new()})));
        assert_eq!(parse_import(" import a.b.c.* from 'a_b-file.fidl'"),
                   Ok(("", ast::Import{ uri: "a_b-file.fidl".to_string(), namespace: "a.b.c.*".to_string()})));
    }

    #[test]
    fn test_interface() {
        let txt = "interface MyInterface { version {major 1 minor 34} }    ";
        let (_, interface) = parse_interface(txt).unwrap();
        if let ModuleContent::Interface(intf) = interface {
            assert_eq!(intf.annotation, None);
            assert_eq!(intf.name, "MyInterface");
            assert_eq!(intf.version, Some((1, 34)));
        }
        else {
            assert!(false, "interface is not ModuleContent::Interface");
        }

        let txt = "<** This is an annotation **> \ninterface Another_Interface\n{ \n}";
        let (_, interface) = parse_interface(txt).unwrap();
        if let ModuleContent::Interface(intf) = interface {
            assert_eq!(intf.annotation, Some(" This is an annotation ".to_string()));
            assert_eq!(intf.name, "Another_Interface");
            assert_eq!(intf.version, None);
        }
        else {
            assert!(false, "interface is not ModuleContent::Interface");
        }
    }

    #[test]
    fn test_string() {
        assert_eq!(parse_string("\"a string \"add"), Ok(("add", "a string ".to_string())));
        assert_eq!(parse_string("\'a string \'add"), Ok(("add", "a string ".to_string())));
    }

    #[test]
    fn test_identifier_ok() {
        assert_eq!(parse_identifier("aSimpleIdentifier"), Ok(("", "aSimpleIdentifier")));
        assert_eq!(parse_identifier("Simple_Identifier9 adfj"), Ok((" adfj", "Simple_Identifier9")));
        assert_eq!(parse_identifier("^_aNew_identity!09adf"), Ok(("!09adf", "^_aNew_identity")));
    }

    #[test]
    fn test_identifier_nok() {
        assert_eq!(parse_identifier(" aSimpleIdentifier"),
                   Err(nom::Err::Error(nom::error::Error::new(" aSimpleIdentifier", nom::error::ErrorKind::RegexpMatch))));
        assert_eq!(parse_identifier("9invalid with number"),
                   Err(nom::Err::Error(nom::error::Error::new("9invalid with number", nom::error::ErrorKind::RegexpMatch))));
        assert_eq!(parse_identifier("!ui ui"),
                   Err(nom::Err::Error(nom::error::Error::new("!ui ui", nom::error::ErrorKind::RegexpMatch))));
    }

    #[test]
    fn test_fqn_ok() {
        assert_eq!(parse_fqn("acad.ad09_.ab"), Ok(("", "acad.ad09_.ab".to_string())));
        assert_eq!(parse_fqn("_903.xaf.Ab9.__holla therest"), Ok((" therest", "_903.xaf.Ab9.__holla".to_string())));
        assert_eq!(parse_fqn("_903 xaf.Ab9.__holla therest"), Ok((" xaf.Ab9.__holla therest", "_903".to_string())));
    }

    #[test]
    fn test_fqn_nok() {
        assert_eq!(parse_fqn("0acad.ad09_.ab"),
                   Err(nom::Err::Error(nom::error::Error::new("0acad.ad09_.ab", nom::error::ErrorKind::RegexpMatch))));
    }

    #[test]
    fn test_package_ok() {
        assert_eq!(parse_package("  package    my.package"), Ok(("", "my.package".to_string())));
        assert_eq!(parse_package("package ^anew.package.p01\nrubbish"), Ok(("\nrubbish", "^anew.package.p01".to_string())));
    }

    #[test]
    fn test_package_nok() {
        assert_eq!(parse_package("  ackage    my.package"),
                   Err(nom::Err::Error(nom::error::Error::new("ackage    my.package", nom::error::ErrorKind::Tag))));
    }
}
