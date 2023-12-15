use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};

use openmina_node_testing::network_debugger::Connection;

// playground processing connections information
fn main() {
    let mut cns = BTreeMap::<u32, Vec<_>>::default();

    let f = BufReader::new(File::open("target/cns").unwrap());
    for x in f.lines() {
        let line = x.unwrap();
        let mut parts = line.split(':');
        let id = parts.next().unwrap().parse::<u32>().unwrap();
        let pos = line.find(':').unwrap();
        let cn_str = &line[(pos + 1)..];
        let cn = serde_json::from_str::<Connection>(cn_str).unwrap();
        cns.entry(cn.info.fd).or_default().push((
            id,
            cn.incoming,
            cn.info.addr,
            cn.timestamp_close,
        ));
    }
    for items in cns.values_mut() {
        items.sort_by(|(_, _, _, a), (_, _, _, b)| a.cmp(b));
    }

    for (id, items) in cns {
        println!("{id}: {items:?}");
    }
}
