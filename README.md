# Cuckoo filter
----

[![Build Status](https://travis-ci.com/dakv/cuckoo.svg?branch=master)](https://travis-ci.com/dakv/cuckoo)
[![Version](https://img.shields.io/crates/v/dakv_cuckoo.svg)](https://crates.io/crates/dakv_cuckoo)
[![Coverage Status](https://coveralls.io/repos/github/dakv/cuckoo/badge.svg?branch=master)](https://coveralls.io/github/dakv/cuckoo?branch=master)


```rust
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

```