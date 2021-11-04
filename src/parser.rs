// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth
use nom::{
    IResult,
    sequence::{tuple, pair},
    branch::{alt},
    bytes::complete::{tag, take_while, take},
    character::complete::{multispace0, multispace1, char, digit1},
    multi::{fold_many0, many0}
};
use super::util::option;
use lazy_static::lazy_static;
use regex::Regex;

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

fn parse_import_from(input: &str) -> IResult<&str, ast::Import> {
    let (r, v) = tuple((
// todo fqn must be extended to ImportedFQN:  FQN ('.' '*')*
        multispace0, tag("import"), multispace1, parse_fqn, multispace1,
        tag("from"), multispace1, parse_string,
    ))(input)?;
    Ok((r, ast::Import{ uri: v.7.to_string(), namespace: v.3.to_string()}))
}

/// Import : 'import' (importedNamespace=ImportedFQN 'from' | 'model') importURI=STRING;
fn parse_import(input: &str) -> IResult<&str, ast::Import> {
    alt((parse_import_from, parse_import_model))(input)
}

/// FAnnotationBlock returns FAnnotationBlock:
/// 	'<**' (elements+=FAnnotation)+ '**>';
fn parse_annotation(input: &str) -> IResult<&str, Option<String>> {
    match nom::sequence::tuple((
        nom::bytes::complete::tag("<**"),
        nom::bytes::complete::take_until("**>"),
        nom::bytes::complete::tag("**>"),
        multispace0,
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
        multispace1, tag("minor"), multispace1, digit1, multispace0, tag("}")
    ))(input) as IResult<&str, (&str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str)>  {
        Ok((r, v)) => Ok((r, Some(( v.6.parse::<u32>().unwrap(), v.10.parse::<u32>().unwrap())))),
        Err(_) => Ok((input, None))
    }
}

enum ModuleContent {
    Interface(ast::Interface),
    TypeCollection(ast::TypeCollection)
}

fn parse_interface(input: &str) -> IResult<&str, ModuleContent> {
    let (r, v) = nom::sequence::tuple((
        parse_annotation, tag("interface"), multispace1,
        parse_identifier, multispace0,
        // todo extends
        // todo manages
        tag("{"), multispace0, parse_version,

        multispace0, tag("}"),
    ))(input)?;
    Ok((r, ModuleContent::Interface( ast::Interface{
        annotation: v.0, name: v.3.to_string(), version: v.7
    })))
}

fn parse_type_collection(input: &str) -> IResult<&str, ModuleContent> {
    let (r, _) = nom::sequence::tuple((
        parse_annotation, multispace1, tag("typeCollection"), multispace0,
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

#[cfg(test)]
mod test {
    use super::*;

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
                   Ok((" \n a new line", ast::Import{ uri: "a_b-file.fidl".to_string(), namespace: String::new()})))
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
