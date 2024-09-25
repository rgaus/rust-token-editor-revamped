mod node_tree;

use crate::node_tree::{
    cursor::{Cursor, CursorSeek},
    node::InMemoryNode,
    utils::Inclusivity,
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

    let parent = InMemoryNode::new_empty();
    let foo = InMemoryNode::new_from_literal("foo");
    let bar = InMemoryNode::new_from_literal("bar");
    let baz = InMemoryNode::new_from_literal("baz ");
    let quux = InMemoryNode::new_from_literal("quux");
    let hello = InMemoryNode::new_from_literal("hello");
    let world = InMemoryNode::new_from_literal("world");

    // // Test 1:
    // // let foo = InMemoryNode::append_child(foo, baz);
    // // let foo = InMemoryNode::append_child(foo, quux);
    // // let hello = InMemoryNode::append_child(hello, world);
    // // let foo = InMemoryNode::append_child(foo, hello);
    // // let parent = InMemoryNode::append_child(parent, foo);
    // // let parent = InMemoryNode::append_child(parent, bar);

    // Test 2:
    InMemoryNode::append_child(&foo, bar);
    InMemoryNode::append_child(&foo, baz);
    InMemoryNode::append_child(&quux, hello);
    InMemoryNode::append_child(&quux, world);

    InMemoryNode::append_child(&parent, foo);
    InMemoryNode::append_child(&parent, quux.clone());

    println!("");
    InMemoryNode::dump(&parent);

    // // Remove test:
    // println!("");
    // InMemoryNode::remove_child_at_index(&parent, 0);

    // // Swap test:
    // let new_child = InMemoryNode::new_from_literal("NEW");
    // InMemoryNode::swap_child_at_index(&parent, 0, new_child);

    // println!("");
    // InMemoryNode::dump(&parent);

    // let results = InMemoryNode::seek_forwards_until(&parent, |_node, _ct| SeekResult::Continue);
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

    let cur = Cursor::new(parent);
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
    // let output = cur.seek_forwards(CursorSeek::AdvanceUntil(Rc::new(|c| {
    //     if c == 'w' { CursorSeekAdvanceUntil::Stop } else { CursorSeekAdvanceUntil::Continue }
    // })));
    // let output = cur.seek_forwards(CursorSeek::advance_until_char_then_stop(' '));
    // let output = cur.seek_forwards(CursorSeek::advance_until_char_then_done(' '));
    let (cur, output) = cur.seek_forwards(CursorSeek::advance_upper_word(Inclusivity::Exclusive));
    // let output = cur.seek_forwards(CursorSeek::advance_lower_word(CursorInclusivity::Exclusive));
    // let output = cur.seek_forwards(CursorSeek::advance_upper_word(CursorInclusivity::Inclusive));
    println!("STRING: {:?}", output);
}
