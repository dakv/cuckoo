use dakv_cuckoo::CuckooFilter;

fn main() {
    let mut cf = CuckooFilter::default();
    let _ = cf.add(b"test");
    assert_eq!(cf.size(), 1);
    assert!(cf.contains(b"test"));
    assert!(cf.delete(b"test"));
    assert_eq!(cf.size(), 0);
    assert!(!cf.contains(b"test"));
}
