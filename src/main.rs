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

    let foo = InMemoryNode::append_child(foo, baz);
    let foo = InMemoryNode::append_child(foo, quux);
    let foo = InMemoryNode::append_child(foo, hello);
    let parent = InMemoryNode::append_child(parent, foo);
    let parent = InMemoryNode::append_child(parent, bar);

    println!("");
    InMemoryNode::dump(&parent);
}
