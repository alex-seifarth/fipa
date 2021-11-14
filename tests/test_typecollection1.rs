// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth

#[test]
fn test_type_collection1() {
    let fidl_text = include_str!("type_collection1.fidl");
    let module_result = fipa::parser::parse_module(&fidl_text);

    assert!(module_result.is_ok());
    let (_, module) = module_result.unwrap();

    // package name
    assert_eq!(module.package, "de.titnc.fidl_test");
    assert_eq!(module.imports.len(), 0);
    assert_eq!(module.type_collections.len(), 2);

    let coll1 = &module.type_collections[0];
    assert_eq!(coll1.annotation, None);
    assert_eq!(coll1.version, Some((2, 14)));
    assert_eq!(coll1.name, Some("MyTypeCollection".to_string()));
    assert_eq!(coll1.types.len(), 4);

    // public typedef Byte is UInt8
    assert_eq!(coll1.types[0], fipa::ast::Type::TypeDef {
       annotation: None, public: true, name: "Byte".to_string(), array: false,
        actual_type: fipa::ast::TypeRef::UInt8 });

    // <** an array of int's **> public array IntArray of Int8
    assert_eq!(coll1.types[1], fipa::ast::Type::Array {
        annotation: Some(" an array of int's ".to_string()), public: true, name: "IntArray".to_string(),
        element_type: fipa::ast::TypeRef::Int8 });

    // public struct AStruct { Byte a_single_byte Boolean flag String explanation }
    assert_eq!(coll1.types[2], fipa::ast::Type::Struct {
        annotation: None, public: true, name: "AStruct".to_string(), extends: None, polymorphic: false,
        fields: vec![
            fipa::ast::Field{ annotation: None, name: "a_single_byte".to_string(), array: false, type_ref: fipa::ast::TypeRef::Derived("Byte".to_string()) },
            fipa::ast::Field{ annotation: None, name: "flag".to_string(), array: false, type_ref: fipa::ast::TypeRef::Boolean },
            fipa::ast::Field{ annotation: None, name: "explanation".to_string(), array: false, type_ref: fipa::ast::TypeRef::String }
    ] });

    // public enumeration Status { NO_SIGNAL, VALUE_1, SIGNAL_UNBEFUELLT = 0x0f, }
    assert_eq!(coll1.types[3], fipa::ast::Type::Enumeration {
        annotation: None, public: true, name: "Status".to_string(), base_type: None,
        enumerators: vec! [
            fipa::ast::Enumerator{ annotation: None, name: "NO_SIGNAL".to_string(), val: None},
            fipa::ast::Enumerator{ annotation: None, name: "VALUE_1".to_string(), val: None },
            fipa::ast::Enumerator{ annotation: None, name: "SIGNAL_UNBEFUELLT".to_string(), val: Some(0x0f)},
    ] });


    let coll2 = &module.type_collections[1];
    assert_eq!(coll2.annotation, Some(" an 'anonymous' type collection ".to_string()));
    assert_eq!(coll2.version, None);
    assert_eq!(coll2.name, None);
    assert_eq!(coll2.types.len(), 0);
}
