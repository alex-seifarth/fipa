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

    /// types defined within the interface
    pub types: Vec<Type>,

    /// broadcasts defined within this interface
    pub broadcasts: Vec<Broadcast>,

    /// methods defined within this interface
    pub methods: Vec<Method>,

    /// Optional base interface identifier
    pub extends: Option<String>,

    /// Optional list of managed interface identifiers
    pub manages: Option<Vec<String>>,
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

/// {FBroadcast} (comment=FAnnotationBlock)?
/// 'broadcast' name=ID (':' selector=ID)? (selective?='selective')? '{' ('out' '{' (outArgs+=FArgument)* '}' )? '}';
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Broadcast {
    pub annotation: Option<String>,
    pub name: String,
    pub selector: Option<String>,
    pub selective: bool,
    pub out_args: Vec<Argument>,
}

///FField: 	(comment=FAnnotationBlock)?  type=FTypeRef (array?='[' ']')? name=ID;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    pub annotation: Option<String>,
    pub name: String,
    pub type_ref: TypeRef,
    pub array: bool,
}

/// FArgument: (comment=FAnnotationBlock)? type=FTypeRef (array?='[' ']')? name=ID;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Argument {
    pub annotation: Option<String>,
    pub type_ref: TypeRef,
    pub array: bool,
    pub name: String,
}

///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Method {
    pub annotation: Option<String>,
    pub name: String,
    pub selector: Option<String>,
    pub fire_and_forget: bool,
    pub in_args: Vec<Argument>,
    pub out_args: Vec<Argument>,
    pub error: Option<MethodErrorSpec>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MethodErrorSpec {
    Reference{annotation: Option<String>, fqn: String},
    EnumerationBody{annotation: Option<String>, extends: Option<TypeRef>, enumerators: Vec<Enumerator>}
}

/// FEnumerator returns FEnumerator:
/// 	(comment=FAnnotationBlock)?
/// 	name=ID ('=' value=AdditiveExpression)?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Enumerator {
    pub annotation: Option<String>,
    pub name: String,
    pub val: Option<u64>,
}

/// FRANCA type collection specification
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeCollection {
    /// Optional annotation associated with the interface
    pub annotation: Option<String>,

    /// name of the interface - must be unique within a package/module
    pub name: Option<String>,

    /// version in from (major, minor)
    pub version: Option<(u32, u32)>,

    /// Types defined within this type collection
    pub types: Vec<Type>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    TypeDef{ annotation: Option<String>, public: bool, name: String, array: bool, actual_type: TypeRef },
    Array{ annotation: Option<String>, public: bool, name: String, element_type: TypeRef },
    Struct{ annotation: Option<String>, public: bool, name: String, polymorphic: bool, extends: Option<String>, fields: Vec<Field>},
    Union{ annotation: Option<String>, public: bool, name: String, base_type: Option<String>, fields: Vec<Field>},
    Map{ annotation: Option<String>, public: bool, name: String, key_type: TypeRef, value_type: TypeRef},
    Enumeration{ annotation: Option<String>, name: String, public: bool, base_type: Option<TypeRef>, enumerators: Vec<Enumerator>}
}
