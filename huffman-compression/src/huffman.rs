use anyhow::Result;
use bytes::Buf;
use std::{
    any::Any,
    collections::{BinaryHeap, HashMap},
    fs::File,
    io::{Read, Write},
    rc::Rc,
};

type HuffNode = Rc<Box<dyn HuffBaseNode>>;

const MAGIC_NUMBER: &[u8; 4] = b"HUFF";

pub trait HuffBaseNode {
    fn is_leaf(&self) -> bool;
    fn weight(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
}

pub struct HuffLeafNode {
    element: char,
    weight: u64,
}

pub struct HuffInternalNode {
    left: HuffNode,
    right: HuffNode,
    weight: u64,
}

impl HuffLeafNode {
    pub fn new(element: char, weight: u64) -> HuffLeafNode {
        HuffLeafNode { element, weight }
    }

    pub fn value(&self) -> char {
        self.element
    }
}

impl HuffBaseNode for HuffLeafNode {
    fn is_leaf(&self) -> bool {
        true
    }

    fn weight(&self) -> u64 {
        self.weight
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl HuffInternalNode {
    pub fn new(left: HuffNode, right: HuffNode, weight: u64) -> HuffInternalNode {
        HuffInternalNode {
            left,
            right,
            weight,
        }
    }

    pub fn left(&self) -> HuffNode {
        self.left.clone()
    }

    pub fn right(&self) -> HuffNode {
        self.right.clone()
    }
}

impl HuffBaseNode for HuffInternalNode {
    fn is_leaf(&self) -> bool {
        false
    }

    fn weight(&self) -> u64 {
        self.weight
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for dyn HuffBaseNode {
    fn eq(&self, other: &Self) -> bool {
        self.weight() == other.weight()
    }
}
impl PartialOrd for dyn HuffBaseNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.weight().cmp(&other.weight()))
    }
}
impl Eq for dyn HuffBaseNode {}
impl Ord for dyn HuffBaseNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct BoxedHuffNode {
    pub inner: HuffNode,
}

impl BoxedHuffNode {
    pub fn new(inner: HuffNode) -> BoxedHuffNode {
        BoxedHuffNode { inner }
    }

    pub fn weight(&self) -> u64 {
        self.inner.weight()
    }

    pub fn is_leaf(&self) -> bool {
        self.inner.is_leaf()
    }
}

impl PartialEq for BoxedHuffNode {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}
impl PartialOrd for BoxedHuffNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.inner.partial_cmp(&self.inner)
    }
}
impl Eq for BoxedHuffNode {}
impl Ord for BoxedHuffNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct HuffmanCompression {
    pub src: String,
    pub dst: String,
}

impl HuffmanCompression {
    pub(crate) fn read(&self) -> Result<HashMap<char, u64>> {
        let mut file = File::open(&self.src)?;
        let mut buf = String::new();
        let mut table = HashMap::new();
        file.read_to_string(&mut buf)?;

        for c in buf.chars() {
            let frequency = table.get(&c).unwrap_or(&0);
            table.insert(c, frequency + 1);
        }

        Ok(table)
    }

    pub(crate) fn build_tree(&self, table: HashMap<char, u64>) -> BoxedHuffNode {
        let mut heap = BinaryHeap::new();

        for entry in table {
            let node = BoxedHuffNode {
                inner: Rc::new(Box::new(HuffLeafNode::new(entry.0, entry.1))),
            };
            heap.push(node);
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();

            let weight = left.weight() + right.weight();
            let left = left.inner.clone();
            let right = right.inner.clone();

            let internal_node = HuffInternalNode::new(left, right, weight);

            heap.push(BoxedHuffNode {
                inner: Rc::new(Box::new(internal_node)),
            });
        }

        heap.pop().unwrap()
    }

    pub(crate) fn generate_huffman_code(&self, root: BoxedHuffNode) -> HashMap<char, String> {
        let mut map: HashMap<char, String> = HashMap::new();
        let mut bits = String::new();
        dfs(root, &mut map, &mut bits);
        map
    }

    pub fn encode(&self) -> Result<()> {
        let frequency_table = self.read()?;
        let root_node = self.build_tree(frequency_table);
        let lookup_table = self.generate_huffman_code(root_node);

        let mut encoded_data = vec![];
        self.write_header(&mut encoded_data, &lookup_table);

        let content = self.encode_content(&lookup_table)?;
        encoded_data.extend(content);

        let mut file = File::create(&self.dst)?;
        file.write_all(&encoded_data)?;
        file.flush()?;

        Ok(())
    }

    pub fn decode(&self) -> Result<()> {
        let mut file = File::open(&self.src)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        if &buf[0..4] != MAGIC_NUMBER {
            println!("not huffman compression file!");
            return Ok(());
        }

        let mut buf = &buf[..];
        buf.get_u32();
        let table_len = buf.get_u32();
        let mut len = 0;
        let mut lookup_table = HashMap::new();

        while len < table_len {
            let key_len = buf.get_u8();
            let key = String::from_utf8(buf[..(key_len as usize)].to_vec())?;
            buf = &buf[key_len as usize..];
            let value_len = buf.get_u8();
            let value = String::from_utf8(buf[..(value_len as usize)].to_vec())?;
            buf = &buf[value_len as usize..];

            let key = key.chars().nth(0).unwrap();
            lookup_table.insert(value, key);
            len += 1 + key_len as u32 + 1 + value_len as u32;
        }

        let content = buf[..].into_iter().map(|c| format!("{:08b}", c)).fold(
            String::new(),
            |mut content, c| {
                content.push_str(&c);
                content
            },
        );

        let mut matched_str = &content[..];
        let mut decoded_data = String::new();

        while matched_str.len() > 0 {
            for (value, c) in &lookup_table {
                if matched_str.starts_with(value) {
                    decoded_data.push(*c);
                    matched_str = &matched_str[value.len()..];
                }
            }
        }

        let mut file = File::create(&self.dst)?;
        file.write_all(decoded_data.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    fn write_header(&self, encoded_data: &mut Vec<u8>, table: &HashMap<char, String>) {
        encoded_data.extend(MAGIC_NUMBER.iter().map(|c| c.clone()).collect::<Vec<_>>());

        let mut buf = vec![];
        for (key, value) in table.iter() {
            let key = key.to_string();
            buf.push(key.len() as u8);
            buf.extend(key.as_bytes());
            buf.push(value.len() as u8);
            buf.extend(value.as_bytes());
        }

        let table_len = buf.len() as u32;
        encoded_data.extend(table_len.to_be_bytes());
        encoded_data.extend(buf);
    }

    fn encode_content(&self, table: &HashMap<char, String>) -> Result<Vec<u8>> {
        let mut file = File::open(&self.src)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let mut encoded_data = String::new();
        for c in buf.chars() {
            encoded_data.push_str(&table[&c]);
        }

        while encoded_data.len() % 8 != 0 {
            encoded_data.push('0');
        }

        Ok(encoded_data
            .chars()
            .collect::<Vec<_>>()
            .chunks(8)
            .map(|c| {
                let binary_str = c.iter().collect::<String>();
                u8::from_str_radix(&binary_str, 2).unwrap()
            })
            .collect::<Vec<u8>>())
    }
}

fn dfs(root: BoxedHuffNode, map: &mut HashMap<char, String>, bits: &mut String) {
    if root.is_leaf() {
        let node = root.inner.as_any().downcast_ref::<HuffLeafNode>().unwrap();
        map.insert(node.value(), bits.to_string());
        return;
    }

    let node = root
        .inner
        .as_any()
        .downcast_ref::<HuffInternalNode>()
        .unwrap();

    bits.push('0');
    dfs(BoxedHuffNode::new(node.left().clone()), map, bits);
    bits.remove(bits.len() - 1);

    bits.push('1');
    dfs(BoxedHuffNode::new(node.right().clone()), map, bits);
    bits.remove(bits.len() - 1);
}
