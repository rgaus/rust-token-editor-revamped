mod languages;
mod node_tree;

use std::{cell::RefCell, rc::Rc};

use pancurses::{curs_set, echo, endwin, initscr, noecho, resize_term, Input};

use crate::node_tree::{
    cursor::{Cursor, CursorSeek, Selection},
    node::{
        InMemoryNode,
        NodeMetadata,
        // NodeSeek,
    },
    utils::{Inclusivity, Newline},
    // utils::Inclusivity, fractional_index::VariableSizeFractionalIndex,
    // fractional_index::FractionalIndex,
};

fn interactive(root: Rc<RefCell<InMemoryNode<languages::typescript::SyntaxKind>>>) {
    let window = initscr();
    window.printw("Hello Rust");
    window.keypad(true);
    noecho();

    window.clear();
    let (start_y, start_x) = window.get_beg_yx();
    let (end_y, end_x) = window.get_max_yx();
    let (width_chars, height_chars) = (end_x - start_x, end_y - start_y);

    // window.draw_box(10, 10);

    let mut cursor = Cursor::new(root.clone());
    let mut status: String;

    {
        let mut selection = cursor.selection();
        selection.secondary = selection
            .secondary
            .seek_forwards(CursorSeek::advance_until_start_end());
        window.mvaddstr(0, 0, selection.literal());
    }

    loop {
        let (rows, cols) = cursor.to_rows_cols();

        status = format!("({rows}, {cols}) {:?}", cursor);
        window.mvaddstr(height_chars - 1, 0, &status);

        window.mv((rows - 1) as i32, (cols - 1) as i32);
        // dbg!(rows, cols);

        window.refresh();
        match window.getch() {
            Some(Input::KeyResize) => {
                resize_term(0, 0);
            }
            Some(Input::Character(x)) if x == 'q' => break,

            Some(Input::Character('l')) => {
                cursor = cursor.seek_forwards(CursorSeek::AdvanceByCharCount(1));
            }
            Some(Input::Character('h')) => {
                cursor = cursor.seek_backwards(CursorSeek::AdvanceByCharCount(1));
            }
            Some(Input::Character('j')) => {
                cursor = cursor.seek_forwards(CursorSeek::AdvanceByLines(1));
            }
            Some(Input::Character('k')) => {
                cursor = cursor.seek_backwards(CursorSeek::AdvanceByLines(1));
            }
            Some(Input::Character('w')) => {
                cursor = cursor.seek_forwards(CursorSeek::forwards_word(false));
            }
            Some(Input::Character('b')) => {
                cursor = cursor.seek_backwards(CursorSeek::forwards_word(false));
            }

            Some(Input::Character('%')) => {
                cursor = cursor.seek_forwards(CursorSeek::advance_until_matching_delimiter(
                    Inclusivity::Inclusive,
                ));
            }

            Some(Input::Character('0')) => {
                cursor = cursor.seek_backwards(CursorSeek::advance_until_line_start());
            }
            Some(Input::Character('^')) => {
                cursor = cursor.seek_backwards(
                    CursorSeek::advance_until_line_start_after_leading_whitespace(),
                );
            }
            Some(Input::Character('$')) => {
                cursor = cursor.seek_forwards(CursorSeek::advance_until_line_end());
            }
            Some(Input::Character('G')) => {
                cursor = cursor.seek_forwards(CursorSeek::advance_until_start_end());
            }
            Some(Input::Character('g')) => {
                cursor = cursor.seek_backwards(CursorSeek::advance_until_start_end());
            }
            _ => (),
        }
    }

    echo();
    endwin();
}

fn main() {
    let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed(
        r#"
        let foo = "brew";
        function main() {
            console.log("hello world");
        }

        function fizbuzz(n) {
            if (n % 2 == 0) {
                return "fizz";
            } else if (n % 3 == 0) {
                return "buzz";
            } else {
                return "fizzbuzz";
            }
        }
    "#,
    );
    // let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed(r#"
    //     let foo = "brew";
    //     function main() {
    //         console.log("hello world");
    //     }
    // "#);
    // let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed(r#"
    //     let foo = "brew";
    //     function main() {
    //         console.log("hello world");
    //     }
    // "#);
    // let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed("console.log(123);");
    // println!("INITIAL: {:?}", Selection::new_across_subtree(&root));

    if std::env::var("INTERACTIVE").unwrap_or("".into()) == "true" {
        interactive(root);
        std::process::exit(0);
    }

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

    // println!("------ ONE ------");
    // let parent = InMemoryNode::<languages::typescript::SyntaxKind>::new_tree_from_literal_in_chunks("foo:bar baz hello world", 4);
    // InMemoryNode::dump(&parent);

    // println!("------");
    // InMemoryNode::insert_child(&parent, InMemoryNode::new_from_literal("NEW!"), 4);
    // InMemoryNode::dump(&parent);
    // println!("------");
    // InMemoryNode::insert_child(&parent.borrow().children[2].clone(), InMemoryNode::new_from_literal("BLEW!"), 0);
    // InMemoryNode::insert_child(&parent.borrow().children[2].clone(), InMemoryNode::new_from_literal("YOO"), 0);
    // InMemoryNode::dump(&parent);
    // println!("------");

    // // let cur = Cursor::new_at(parent.borrow().children[2].clone(), 0);
    // let cur = Cursor::new_at(parent.clone(), 0);
    // let mut selection = cur.selection();
    // selection.set_primary(selection.primary.seek_forwards(CursorSeek::AdvanceByCharCount(10)));
    // // selection.set_primary(selection.primary.seek_forwards(CursorSeek::advance_lower_word(Inclusivity::Inclusive)));
    // // selection.set_primary(selection.primary.seek_forwards(CursorSeek::advance_lower_word(Inclusivity::Exclusive)));
    // println!("SELECTION: {selection:?}");
    // selection.delete().unwrap();

    // println!("------");
    // // InMemoryNode::dump(&parent);
    // println!("{:?}", Selection::new_across_subtree(&parent));

    // println!("------ END ONE ------");

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

    // rslint_example::main();
    println!("------ TWO ------");
    // let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed(r#"
    //     let foo = "brew";
    //     function main() {
    //         console.log("hello world");
    //     }

    //     function fizbuzz(n) {
    //         if (n % 2 == 0) {
    //             return "fizz";
    //         } else if (n % 3 == 0) {
    //             return "buzz";
    //         } else {
    //             return "fizzbuzz";
    //         }
    //     }
    // "#);
    // // let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed(r#"
    // //     let foo = "brew";
    // //     function main() {
    // //         console.log("hello world");
    // //     }
    // // "#);
    // // let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed(r#"
    // //     let foo = "brew";
    // //     function main() {
    // //         console.log("hello world");
    // //     }
    // // "#);
    // // let root = InMemoryNode::<languages::typescript::SyntaxKind>::new_from_parsed("console.log(123);");
    // InMemoryNode::dump(&root);
    // // println!("INITIAL: {:?}", Selection::new_across_subtree(&root));

    let mut selection = Cursor::new(root.clone()).selection();
    // selection.set_primary(selection.primary.seek_forwards(CursorSeek::advance_until_char_then_stop('}', Newline::Ignore)));
    selection.set_primary(
        selection
            .primary
            .seek_forwards(CursorSeek::AdvanceByCharCount(3)),
    );
    println!("ROWS COLS: {:?}", selection.primary.to_rows_cols());
    // selection.set_primary(selection.primary.seek_forwards(CursorSeek::advance_until_matching_delimiter(Inclusivity::Inclusive)));
    // selection.set_primary(selection.primary.seek_forwards(CursorSeek::AdvanceByCharCount(10)));
    // selection.set_secondary(selection.secondary.seek_forwards(CursorSeek::AdvanceByCharCount(18)));
    // selection.set_secondary(selection.secondary.seek_forwards(CursorSeek::AdvanceByCharCount(9)));
    // selection.set_secondary(selection.secondary.seek_forwards(CursorSeek::AdvanceByCharCount(10)));
    // selection.set_secondary(selection.secondary.seek_forwards(CursorSeek::AdvanceByLines(2)));
    println!("--------");
    // selection.set_secondary(selection.secondary.seek_backwards(CursorSeek::AdvanceByLines(2)));
    // selection.set_secondary(selection.secondary.seek_forwards(CursorSeek::AdvanceByCharCount(3)));
    // selection.set_secondary(selection.secondary.seek_forwards(CursorSeek::advance_until_char_then_done('"', Newline::ShouldTerminate)));
    // selection.set_secondary(selection.secondary.seek_backwards(CursorSeek::advance_until_line_start()));
    println!("PRE: {:?}\n", selection);
    // // selection.delete_raw().unwrap();
    // selection.delete().unwrap();
    // println!("-------");
    // println!("POST: {:?}", Selection::new_across_subtree(&root));
    // InMemoryNode::dump(&root);
    // // InMemoryNode::dump_trace(&root);

    // let mut selection = Cursor::new(root.clone()).selection();
    // selection.set_secondary(selection.secondary.seek_forwards_until(|_n, _ct| CursorSeek::Continue));
    // println!("RESULT: {:?} {:?}", selection, selection.secondary.to_rows_cols());

    // println!("NEW: {:?}", Cursor::new_at_rows_cols(root.clone(), (15, 1)));

    println!("------ END TWO ------");
    // println!("-------");
    // let a = VariableSizeFractionalIndex::of(vec![252]);
    // let b = VariableSizeFractionalIndex::of(vec![255]);
    // // let c = VariableSizeFractionalIndex::generate(a.clone(), b.clone());
    // // println!("A: {a:?}");
    // // println!("C: {c:?}");
    // // println!("B: {b:?}");
    // // println!("{:?}", a < c);

    // let mut seq = VariableSizeFractionalIndex::eqidistributed_sequence(a, b, 10);
    // for _ in 0..10 {
    //     println!("=> {:?}", seq.next());
    // }
}
