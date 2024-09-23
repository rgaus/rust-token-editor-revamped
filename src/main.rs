mod node;
mod node_debug_validators;

use node::InMemoryNode;

fn main() {
    let parent = InMemoryNode::new_empty();
    let foo = InMemoryNode::new_from_literal("foo");
    let bar = InMemoryNode::new_from_literal("bar");
    let baz = InMemoryNode::new_from_literal("baz");
    let quux = InMemoryNode::new_from_literal("quux");
    let hello = InMemoryNode::new_from_literal("hello");
    let world = InMemoryNode::new_from_literal("world");

    // Test 1:
    // let foo = InMemoryNode::append_child(foo, baz);
    // let foo = InMemoryNode::append_child(foo, quux);
    // let hello = InMemoryNode::append_child(hello, world);
    // let foo = InMemoryNode::append_child(foo, hello);
    // let parent = InMemoryNode::append_child(parent, foo);
    // let parent = InMemoryNode::append_child(parent, bar);

    // Test 2:
    InMemoryNode::append_child(&foo, bar);
    InMemoryNode::append_child(&foo, baz);
    InMemoryNode::append_child(&quux, hello);
    InMemoryNode::append_child(&quux, world);

    InMemoryNode::append_child(&parent, foo);
    InMemoryNode::append_child(&parent, quux);

    println!("");
    InMemoryNode::dump(&parent);

    println!("");
    InMemoryNode::remove_child_at_index(&parent, 0);

    println!("");
    InMemoryNode::dump(&parent);
}
