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
    assert_eq!(module.interfaces.len(), 1);
}