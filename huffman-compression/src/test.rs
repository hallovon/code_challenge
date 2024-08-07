use std::collections::HashMap;

use huffman::{HuffInternalNode, HuffLeafNode};

use super::*;

#[allow(dead_code)]
fn init_huffman_compression() -> HuffmanCompression {
    HuffmanCompression {
        src: "./test.txt".to_string(),
        dst: "huffman.bin".to_string(),
    }
}

#[test]
fn test_table_character_count() {
    let huffman = init_huffman_compression();
    let table = huffman.read().unwrap();
    assert_eq!(table.get(&'X').unwrap(), &333);
    assert_eq!(table.get(&'t').unwrap(), &223000);
}

#[test]
fn test_build_huffman_tree() {
    let huffman = init_huffman_compression();

    let mut table = HashMap::new();
    table.insert('C', 32);
    table.insert('D', 42);
    table.insert('E', 120);
    table.insert('K', 7);
    table.insert('L', 42);
    table.insert('M', 24);
    table.insert('U', 37);
    table.insert('Z', 2);

    let root = huffman.build_tree(table);
    assert!(!root.is_leaf());
    assert_eq!(root.weight(), 306);
    let root = root.inner.as_any().downcast_ref::<HuffInternalNode>();
    assert!(root.is_some());
    let root = root.unwrap();

    let left_leaf = root.left();
    assert!(left_leaf.is_leaf());
    assert_eq!(left_leaf.weight(), 120);
    let left_leaf = left_leaf.as_any().downcast_ref::<HuffLeafNode>();
    assert!(left_leaf.is_some());
    let left_leaf = left_leaf.unwrap();
    assert_eq!(left_leaf.value(), 'E');

    let right_node = root.right();
    assert!(!right_node.is_leaf());
    assert_eq!(right_node.weight(), 186);
    let right_node = right_node.as_any().downcast_ref::<HuffInternalNode>();
    assert!(right_node.is_some());
    let _right_node = right_node.unwrap();
}

#[test]
fn test_huffman_lookup_table() {
    let huffman = init_huffman_compression();

    let mut table = HashMap::new();
    table.insert('C', 32);
    table.insert('D', 42);
    table.insert('E', 120);
    table.insert('K', 7);
    table.insert('L', 42);
    table.insert('M', 24);
    table.insert('U', 37);
    table.insert('Z', 2);

    let mut lookup_table = HashMap::new();
    lookup_table.insert('C', "1110".to_owned());
    lookup_table.insert('D', "101".to_owned());
    lookup_table.insert('E', "0".to_owned());
    lookup_table.insert('K', "111101".to_owned());
    lookup_table.insert('L', "110".to_owned());
    lookup_table.insert('M', "11111".to_owned());
    lookup_table.insert('U', "100".to_owned());
    lookup_table.insert('Z', "111100".to_owned());

    let root = huffman.build_tree(table);
    let res = huffman.generate_huffman_code(root);

    assert_eq!(lookup_table, res);
}

#[test]
fn test_encode_file() {
    let huffman = init_huffman_compression();
    huffman.encode().unwrap();
}
