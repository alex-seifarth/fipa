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
    assert_eq!(vehicle_status.attributes.len(), 0);
}