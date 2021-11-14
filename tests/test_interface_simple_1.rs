// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth

use fipa;

#[test]
fn test_interface_simple_1a() {
    let fidl_text = include_str!("interface1a.fidl");

    let module_result = fipa::parser::parse_module(&fidl_text);
    assert!(module_result.is_ok());

    let (_, module) = module_result.unwrap();

    // package name
    assert_eq!(module.package, "de.titnc.my_test.package");

    // imports
    assert_eq!(module.imports.len(), 2);

    assert_eq!(module.imports[0].namespace, "de.titnc.my_test.sub_package");
    assert_eq!(module.imports[0].uri, "types.fidl");

    assert!(module.imports[1].namespace.is_empty());
    assert_eq!(module.imports[1].uri, "a-very-nice-model");

    // interface
    assert_eq!(module.interfaces.len(), 2);

    // MyInterface
    let my_interface = &module.interfaces[0];
    assert_eq!(my_interface.annotation, None);
    assert_eq!(my_interface.name, "MyInterface");
    assert_eq!(my_interface.version, None);
    assert_eq!(my_interface.attributes.len(), 3);

    //     attribute Int8 counter readonly
    assert_eq!(my_interface.attributes[0].annotation, None);
    assert_eq!(my_interface.attributes[0].name, "counter");
    assert_eq!(my_interface.attributes[0].array, false);
    assert_eq!(my_interface.attributes[0].no_read, false);
    assert_eq!(my_interface.attributes[0].read_only, true);
    assert_eq!(my_interface.attributes[0].no_subscription, false);
    assert_eq!(my_interface.attributes[0].type_ref, fipa::ast::TypeRef::Int8);

    //     attribute Boolean active
    assert_eq!(my_interface.attributes[1].annotation, None);
    assert_eq!(my_interface.attributes[1].name, "active");
    assert_eq!(my_interface.attributes[1].array, false);
    assert_eq!(my_interface.attributes[1].no_read, false);
    assert_eq!(my_interface.attributes[1].read_only, false);
    assert_eq!(my_interface.attributes[1].no_subscription, false);
    assert_eq!(my_interface.attributes[1].type_ref, fipa::ast::TypeRef::Boolean);

    //     attribute MyType[] headlines noSubscription
    assert_eq!(my_interface.attributes[2].annotation, Some(" This is an attribute with annotation ".to_string()));
    assert_eq!(my_interface.attributes[2].name, "headlines");
    assert_eq!(my_interface.attributes[2].array, true);
    assert_eq!(my_interface.attributes[2].no_read, false);
    assert_eq!(my_interface.attributes[2].read_only, false);
    assert_eq!(my_interface.attributes[2].no_subscription, true);
    assert_eq!(my_interface.attributes[2].type_ref, fipa::ast::TypeRef::Derived("MyType".to_string()));

    // VehicleStatus
    let vehicle_status = &module.interfaces[1];
    assert!(vehicle_status.annotation.is_some());
    assert_eq!(vehicle_status.name, "VehicleStatus");
    assert_eq!(vehicle_status.version, Some((2, 1)));
    assert_eq!(vehicle_status.attributes.len(), 1);
    assert_eq!(vehicle_status.types.len(), 2);

    assert_eq!(vehicle_status.attributes[0], fipa::ast::Attribute{
        annotation: None, name: "actualFesMode".to_string(), array: false, read_only: true,
        no_read: false, no_subscription: false, type_ref: fipa::ast::TypeRef::Derived("FesMode".to_string()) });

    assert_eq!(vehicle_status.types[0], fipa::ast::Type::TypeDef {
        annotation: None, name: "Flag".to_string(), public: false, array: false,
        actual_type: fipa::ast::TypeRef::Boolean });
    assert_eq!(vehicle_status.types[1], fipa::ast::Type::Enumeration {
        annotation: None, name: "FesMode".to_string(), public: true, base_type: None,
        enumerators: vec![
            fipa::ast::Enumerator{ annotation: None, name: "SPORT_INDIVIDUAL".to_string(), val: Some(1) },
            fipa::ast::Enumerator{ annotation: None, name: "COMFORT".to_string(), val: Some(2) },
            fipa::ast::Enumerator{ annotation: None, name: "ECO".to_string(), val: Some(4) },
    ] });

    assert_eq!(vehicle_status.broadcasts.len(), 1);
    assert_eq!(vehicle_status.broadcasts[0], fipa::ast::Broadcast{
        annotation: Some(" a pure event -> broadcast\n    ".to_string()), selector: None, selective: false,
        name: "ZeroEmmissionZoneBorder".to_string(), out_args: vec![
            fipa::ast::Argument{ annotation: None, array: false, name: "zoneEntered".to_string(), type_ref: fipa::ast::TypeRef::Boolean},
            fipa::ast::Argument{ annotation: None, array: false, name: "zoneID".to_string(), type_ref: fipa::ast::TypeRef::Int8 },
        ]
    });

    assert_eq!(vehicle_status.methods.len(), 1);
    assert_eq!(vehicle_status.methods[0], fipa::ast::Method {
        annotation: None, name: "setActiveStatistics".to_string(), fire_and_forget: true, selector: None,
        in_args: vec![
            fipa::ast::Argument{annotation: None, name: "activeStatistics".to_string(), array: false,
                type_ref: fipa::ast::TypeRef::Derived("StatisticsType".to_string())},
            fipa::ast::Argument{annotation: None, name: "resetStatistics".to_string(), array: false,
                type_ref: fipa::ast::TypeRef::Boolean}
        ],
        out_args: Vec::new(), error: None
    });
}