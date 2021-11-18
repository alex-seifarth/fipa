// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth

use fipa;

#[tokio::test]
async fn test_parser() {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/data");

    let fidls = vec![d.join("Service1.fidl"), d.join("Service2.fidl")];
    let search_dir = vec![d.join("common")];
    let max_import_nesting = 256usize;

    let (modules, errors) =
        fipa::compiler::parse_fidls(&fidls, &search_dir, max_import_nesting, true).await;
    assert!(errors.is_empty());
    assert_eq!(modules.len(), 4);



}