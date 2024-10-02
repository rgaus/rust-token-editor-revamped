mod node_tree;

use node_tree::cursor::Selection;

use crate::node_tree::{
    cursor::{Cursor, CursorSeek},
    node::{InMemoryNode, NodeSeek},
    utils::Inclusivity,
    fractional_index::FractionalIndex,
};

fn main() {
    // let foo = mini_js::parse_string(r#"
    //     {
    //         foo
    //         {
    //             bar
    //         }
    //     }
    // "#);
    // println!("");
    // InMemoryNode::dump(&foo);

    // let parent = InMemoryNode::new_empty();
    // let foo = InMemoryNode::new_from_literal("foo");
    // let bar = InMemoryNode::new_from_literal("bar");
    // let baz = InMemoryNode::new_from_literal("baz ");
    // let quux = InMemoryNode::new_from_literal("quux");
    // let hello = InMemoryNode::new_from_literal("hello");
    // let world = InMemoryNode::new_from_literal("world");

    // // Test 1:
    // // let foo = InMemoryNode::append_child(foo, baz);
    // // let foo = InMemoryNode::append_child(foo, quux);
    // // let hello = InMemoryNode::append_child(hello, world);
    // // let foo = InMemoryNode::append_child(foo, hello);
    // // let parent = InMemoryNode::append_child(parent, foo);
    // // let parent = InMemoryNode::append_child(parent, bar);

    // // Test 2:
    // InMemoryNode::append_child(&foo, bar);
    // InMemoryNode::append_child(&foo, baz);
    // InMemoryNode::append_child(&quux, hello);
    // InMemoryNode::append_child(&quux, world);

    // InMemoryNode::append_child(&parent, foo);
    // InMemoryNode::append_child(&parent, quux.clone());

    // println!("");
    // InMemoryNode::dump(&parent);

    // // Remove test:
    // println!("");
    // InMemoryNode::remove_child_at_index(&parent, 0);

    // // Swap test:
    // let new_child = InMemoryNode::new_from_literal("NEW");
    // InMemoryNode::swap_child_at_index(&parent, 0, new_child);

    // println!("");
    // InMemoryNode::dump(&parent);

    // let results = InMemoryNode::seek_forwards_until(&parent, |node, _ct| NodeSeek::Continue(InMemoryNode::literal(node)));
    // println!("RESULT: {:?}", results);

    // let results = InMemoryNode::seek_backwards_until(&quux, |node, _ct| NodeSeek::Continue(InMemoryNode::literal(node)));
    // println!("RESULT: {:?}", results);

    // let results = InMemoryNode::seek_forwards_until(&parent, |_node, _ct| NodeSeek::Continue);
    // let results = InMemoryNode::seek_forwards_until(&parent, |node, ct| {
    //     if ct < 3 {
    //         NodeSeek::Continue(node.clone())
    //     } else {
    //         NodeSeek::Stop
    //     }
    // });
    // // println!("RESULTS: {:?}", results);
    // let string = results.fold("".into(), |acc, node| format!("{acc} {:?}", node.borrow().metadata));
    // println!("STRING: {:?}", string);

    // let output = cur.seek_forwards_until(|_character, ct| {
    //     if ct < 5 {
    //         CursorSeek::Continue
    //     } else if ct == 5 {
    //         CursorSeek::AdvanceByCharCount(1)
    //     } else {
    //         CursorSeek::Done
    //     }
    // });
    // let output = cur.seek_forwards(CursorSeek::AdvanceByCharCount(5));
    // let output = cur.seek_forwards(CursorSeek::advance_until(|c| {
    //     if c == 'w' { CursorSeekAdvanceUntil::Stop } else { CursorSeekAdvanceUntil::Continue }
    // }));
    // let output = cur.seek_forwards(CursorSeek::advance_until_char_then_stop(' '));
    // let output = cur.seek_forwards(CursorSeek::advance_until_char_then_done(' '));
    // let (cur, output) = cur.seek_forwards(CursorSeek::advance_upper_word(Inclusivity::Exclusive));
    // let output = cur.seek_forwards(CursorSeek::advance_lower_word(CursorInclusivity::Exclusive));
    // let output = cur.seek_forwards(CursorSeek::advance_upper_word(CursorInclusivity::Inclusive));
    // let (cur, output) = cur.seek_backwards_until(|_character, ct| {
    //     if ct < 3 {
    //         CursorSeek::Continue
    //     } else if ct == 3 {
    //         CursorSeek::AdvanceByCharCount(2)
    //     } else {
    //         CursorSeek::Done
    //     }
    // });

    let parent = InMemoryNode::new_tree_from_literal_in_chunks("foo:bar baz hello world", 4);
    InMemoryNode::dump(&parent);

    println!("------");
    InMemoryNode::insert_child(&parent, InMemoryNode::new_from_literal("NEW!"), 4);
    InMemoryNode::dump(&parent);
    println!("------");
    InMemoryNode::insert_child(&parent.borrow().children[2].clone(), InMemoryNode::new_from_literal("BLEW!"), 0);
    InMemoryNode::insert_child(&parent.borrow().children[2].clone(), InMemoryNode::new_from_literal("YOO"), 0);
    InMemoryNode::dump(&parent);

    // let cur = Cursor::new_at(parent, 0);
    // // let cur = Cursor::new(parent);
    // // let (cur, output) = cur.seek_forwards(CursorSeek::AdvanceByCharCount(10));
    // // println!("FORWARDS: {:?} {:?}\n", cur, output);
    // // let (cur, output) = cur.seek_forwards(CursorSeek::advance_lower_word(inclusivity));
    // let inclusivity = Inclusivity::Inclusive;
    // let (cur, output) = cur.seek_forwards(CursorSeek::advance_lower_word(inclusivity));
    // println!("FORWARDS: {:?} {:?}", cur, output);
    // // let (cur, output) = cur.seek_backwards(CursorSeek::advance_lower_word(inclusivity));
    // // let (cur, output) = cur.seek_backwards(CursorSeek::AdvanceByCharCount(5));
    // // println!("BACKWARDS: {:?} {:?}", cur, output);

    // // let mut selection = Selection::new_at(parent.clone(), 0);
    // let mut selection = Selection::new_at(parent.borrow().children[2].clone(), 0);
    // selection.set_secondary(
    //     // selection.secondary.seek_forwards(CursorSeek::advance_lower_word(Inclusivity::Exclusive))
    //     selection.secondary.seek_forwards(CursorSeek::AdvanceByCharCount(10))
    // );
    // println!("SELECTION: {:?}", selection);
    // selection.delete().expect("Error calling selection.delete(): ");
    // InMemoryNode::dump(&parent);

    // InMemoryNode::remove_nodes_sequentially_until(&parent, Inclusivity::Exclusive, |node, ct| {
    //     if ct > 3 {
    //         NodeSeek::Done(node.clone())
    //     } else {
    //         NodeSeek::Continue(node.clone())
    //     }
    // });
    // InMemoryNode::dump(&parent);

    // println!("");
    // println!("");
    // let a = FractionalIndex::start();
    // let b = FractionalIndex::generate_or_fallback(Some(a), None);
    // let c = FractionalIndex::generate_or_fallback(Some(a), Some(b));
    // let d = FractionalIndex::generate_or_fallback(Some(a), Some(c));

    // println!("{a} {b} {c} {d}");
}
