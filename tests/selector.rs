extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use common::{read_json, setup};
use jsonpath::{Parser, Selector, SelectorMut};
use serde_json::Value;

mod common;

#[test]
fn selector_mut() {
    setup();

    let mut selector_mut = SelectorMut::default();

    let mut nums = Vec::new();
    let result = selector_mut
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(read_json("./benchmark/example.json"))
        .replace_with(&mut |v| {
            if let Value::Number(n) = v {
                nums.push(n.as_f64().unwrap());
            }
            Some(Value::String("a".to_string()))
        })
        .unwrap()
        .take()
        .unwrap();

    assert_eq!(
        nums,
        vec![8.95_f64, 12.99_f64, 8.99_f64, 22.99_f64, 19.95_f64]
    );

    let mut selector = Selector::default();
    let result = selector
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(&result)
        .select()
        .unwrap();

    assert_eq!(
        vec![
            &json!("a"),
            &json!("a"),
            &json!("a"),
            &json!("a"),
            &json!("a")
        ],
        result
    );
}

#[test]
fn selector_node_ref() {
    let node = Parser::compile("$.*").unwrap();
    let mut selector = Selector::default();
    selector.compiled_path(&node);
    assert!(std::ptr::eq(selector.node_ref().unwrap(), &node));
}

#[test]
fn selector_delete() {
    setup();

    let mut selector_mut = SelectorMut::default();

    let result = selector_mut
        .str_path(r#"$.store..price[?(@>13)]"#)
        .unwrap()
        .value(read_json("./benchmark/example.json"))
        .delete()
        .unwrap()
        .take()
        .unwrap();

    let mut selector = Selector::default();
    let result = selector
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(&result)
        .select()
        .unwrap();

    assert_eq!(
        result,
        vec![
            &json!(8.95),
            &json!(12.99),
            &json!(8.99),
            &Value::Null,
            &Value::Null
        ]
    );
}

#[test]
fn selector_remove() {
    setup();

    let mut selector_mut = SelectorMut::default();

    let result = selector_mut
        .str_path(r#"$.store..price[?(@>13)]"#)
        .unwrap()
        .value(read_json("./benchmark/example.json"))
        .remove()
        .unwrap()
        .take()
        .unwrap();

    let mut selector = Selector::default();
    let result = selector
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(&result)
        .select()
        .unwrap();

    assert_eq!(result, vec![&json!(8.95), &json!(12.99), &json!(8.99)]);
}

#[test]
fn iter_test() {
    use std::rc::Rc;

    struct T {
        pub value: Rc<Value>,
        pub tt: i32,
    }

    let t = Value::Array(vec![
        Value::String(String::from("vv")),
        Value::Array(vec![]),
    ]);
    let t1 = T {
        value: Rc::new(t.clone()),
        tt: 1,
    };
    let t2 = T {
        value: Rc::new(t.clone()),
        tt: 1,
    };

    let values = vec![t1, t2];

    let mut selector = Selector::default();

    let path = "$.[0]";
    let result = selector
        .str_path(path)
        .unwrap()
        .values_iter(values.iter().map(|v| v.value.as_ref()))
        .select()
        .unwrap();

    assert_eq!(result, vec![&json!(["vv", []])]);

    let result = jsonpath::select_with_iter(values.iter().map(|v| v.value.as_ref()), path).unwrap();
    assert_eq!(result.0, vec![&json!(["vv", []])]);
    assert_eq!(result.1, vec![0]);

    let values = vec![json!([1, 2, 3, 4]), json!([2, 2]), json!(3)];

    let mut selector = Selector::default();
    let result = selector
        .str_path(path)
        .unwrap()
        .values_iter(values.iter())
        .select()
        .unwrap();

    assert_eq!(result, vec![&json!([1, 2, 3, 4])]);
}

#[test]
fn test_not_found_by_index() {
    let array = vec![1, 2, 3, 4, 5];
    let haystack = json!(array);
    let result = jsonpath::select(&haystack, "$.[6]");
    assert!(result.is_err());
}

#[test]
fn test_not_found_by_key() {
    let haystack = json!({"asd": 1});
    let result = jsonpath::select(&haystack, "$.aaa");
    assert!(result.is_err());
}
