use roaring::RoaringTreemap;
use std::iter::FromIterator;

fn serialize_deserialize(dataset: Vec<u64>) {
    let rb = RoaringTreemap::from_iter(dataset);

    let mut buffer = vec![];
    rb.serialize_into(&mut buffer).unwrap();

    assert_eq!(buffer.len(), rb.serialized_size());

    let new_rb = RoaringTreemap::deserialize_from(&mut &buffer[..]).unwrap();

    assert_eq!(rb, new_rb);
}

#[test]
fn serialization_of_empty_treemap() {
    serialize_deserialize(vec![])
}

#[test]
fn serialization_basic() {
    serialize_deserialize(vec![1, 2, 3, 4, 5, 100, 1000])
}
