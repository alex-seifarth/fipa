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
}

/// FRANCA type collection specification
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeCollection {

}

