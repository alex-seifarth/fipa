// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth

/// A module corresponds to a single FIDL file
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module {
    /// Package the module defines.
    pub package: String,

    /// Imports of other namespaces and modules
    pub imports: Vec<Import>,

    /// Defined interfaces within the module
    pub interfaces: Vec<Interface>,

    /// Defined type collections within the module
    pub type_collections: Vec<TypeCollection>
}

/// FRANCA import used by a module
/// An import can either import a full module (e.g. another FIDL file) or a namespace from a module
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Import {
    /// Imported namespace, maybe empty if a full module is imported
    pub namespace: String,

    /// URI (e.g. FIDL file name) of an imported module
    pub uri: String,
}

/// FRANCA interface specification
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Interface {
    /// Optional annotation associated with the interface
    pub annotation: Option<String>,

    /// name of the interface - must be unique within a package/module
    pub name: String,

    /// version in from (major, minor)
    pub version: Option<(u32, u32)>,

    /// attributes defined within the interface
    pub attributes: Vec<Attribute>,
}

/// Type reference that may reference a custom (derived) type by its name or FQN
/// or one of the built-in types.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeRef {
    /// Custom type identified by its name or fully qualified name.
    Derived(String),
    Undefined,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Boolean,
    String,
    Float,
    Double,
    ByteBuffer,

    /// Min-Max value range
    IntegerInterval(Option<isize>, Option<isize>)
}

/// FRANCA attribute specification within an interface
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Attribute {
    pub annotation: Option<String>,
    pub name: String,
    pub array: bool,
    pub read_only: bool,
    pub no_subscription: bool,
    pub no_read: bool,
    pub type_ref: TypeRef,
}

///FField: 	(comment=FAnnotationBlock)?  type=FTypeRef (array?='[' ']')? name=ID;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    pub annotation: Option<String>,
    pub name: String,
    pub type_ref: TypeRef,
    pub array: bool,
}

/// FRANCA type collection specification
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeCollection {

}


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    TypeDef{ annotation: Option<String>, public: bool, name: String, array: bool, actual_type: TypeRef },
    Array{ annotation: Option<String>, public: bool, name: String, element_type: TypeRef },
    Struct{ annotation: Option<String>, public: bool, name: String, polymorphic: bool, extends: Option<String>, fields: Vec<Field>},
    Union{ annotation: Option<String>, public: bool, name: String, base_type: Option<String>, fields: Vec<Field>},
    Enumeration{name: String, public: bool, }
}
