mod node_tree;
mod rslint_example;

use std::{rc::Rc, cell::RefCell};

use colored::{ColoredString, Colorize};
use node_tree::{
    // cursor::Selection,
    node::TokenKindTrait,
};
use rslint_example::convert_rslint_syntaxnode_to_inmemorynode;
use rslint_parser::{parse_text, SyntaxKind};

use crate::node_tree::{
    cursor::{Cursor, CursorSeek},
    node::{
        InMemoryNode,
        // NodeSeek,
    }, utils::Inclusivity,
    // utils::Inclusivity, fractional_index::VariableSizeFractionalIndex,
    // fractional_index::FractionalIndex,
};

/// rslint_parser::SyntaxKind is an enum from rslint_parser (javascript parser) which contains all
/// output tokens.
///
/// Implement TokenKindTrait on this enum so that this SyntaxKind enum can be used as a generic
/// parameter to InMemoryNode (which allows InMemoryNode to store nodes of this type)
impl TokenKindTrait for SyntaxKind {
    fn apply_debug_syntax_color(text: String, mut ancestry: std::vec::IntoIter<SyntaxKind>) -> ColoredString {
        let (Some(kind), Some(parent_kind)) = (ancestry.next(), ancestry.next()) else {
            return text.into();
        };

        // Some bespoke syntax highlighting rules:
        if parent_kind == SyntaxKind::VAR_DECL && kind == SyntaxKind::IDENT {
            return ColoredString::from(text).bold().red();
        };
        if parent_kind == SyntaxKind::FN_DECL && kind == SyntaxKind::FUNCTION_KW {
            return ColoredString::from(text).bold().blue();
        };
        if parent_kind == SyntaxKind::LITERAL && kind == SyntaxKind::STRING {
            return ColoredString::from(text).green();
        };
        if kind == SyntaxKind::COMMENT {
            return ColoredString::from(text).bright_black();
        };

        match kind {
            SyntaxKind::TOMBSTONE => text.into(),
            SyntaxKind::EOF => text.into(),
            SyntaxKind::SEMICOLON => text.into(),
            SyntaxKind::COMMA => text.into(),
            SyntaxKind::L_PAREN => text.into(),
            SyntaxKind::R_PAREN => text.into(),
            SyntaxKind::L_CURLY => text.into(),
            SyntaxKind::R_CURLY => text.into(),
            SyntaxKind::L_BRACK => text.into(),
            SyntaxKind::R_BRACK => text.into(),
            SyntaxKind::L_ANGLE => text.into(),
            SyntaxKind::R_ANGLE => text.into(),
            SyntaxKind::TILDE => text.into(),
            SyntaxKind::QUESTION => text.into(),
            SyntaxKind::QUESTION2 => text.into(),
            SyntaxKind::QUESTIONDOT => text.into(),
            SyntaxKind::AMP => text.into(),
            SyntaxKind::PIPE => text.into(),
            SyntaxKind::PLUS => text.into(),
            SyntaxKind::PLUS2 => text.into(),
            SyntaxKind::STAR => text.into(),
            SyntaxKind::STAR2 => text.into(),
            SyntaxKind::SLASH => text.into(),
            SyntaxKind::CARET => text.into(),
            SyntaxKind::PERCENT => text.into(),
            SyntaxKind::DOT => text.into(),
            SyntaxKind::DOT2 => text.into(),
            SyntaxKind::COLON => text.into(),
            SyntaxKind::EQ => text.into(),
            SyntaxKind::EQ2 => text.into(),
            SyntaxKind::EQ3 => text.into(),
            SyntaxKind::FAT_ARROW => text.into(),
            SyntaxKind::BANG => text.into(),
            SyntaxKind::NEQ => text.into(),
            SyntaxKind::NEQ2 => text.into(),
            SyntaxKind::MINUS => text.into(),
            SyntaxKind::MINUS2 => text.into(),
            SyntaxKind::LTEQ => text.into(),
            SyntaxKind::GTEQ => text.into(),
            SyntaxKind::PLUSEQ => text.into(),
            SyntaxKind::MINUSEQ => text.into(),
            SyntaxKind::PIPEEQ => text.into(),
            SyntaxKind::AMPEQ => text.into(),
            SyntaxKind::CARETEQ => text.into(),
            SyntaxKind::SLASHEQ => text.into(),
            SyntaxKind::STAREQ => text.into(),
            SyntaxKind::PERCENTEQ => text.into(),
            SyntaxKind::AMP2 => text.into(),
            SyntaxKind::PIPE2 => text.into(),
            SyntaxKind::SHL => text.into(),
            SyntaxKind::SHR => text.into(),
            SyntaxKind::USHR => text.into(),
            SyntaxKind::SHLEQ => text.into(),
            SyntaxKind::SHREQ => text.into(),
            SyntaxKind::USHREQ => text.into(),
            SyntaxKind::AMP2EQ => text.into(),
            SyntaxKind::PIPE2EQ => text.into(),
            SyntaxKind::STAR2EQ => text.into(),
            SyntaxKind::QUESTION2EQ => text.into(),
            SyntaxKind::AT => text.into(),
            SyntaxKind::AWAIT_KW => text.into(),
            SyntaxKind::BREAK_KW => text.into(),
            SyntaxKind::CASE_KW => text.into(),
            SyntaxKind::CATCH_KW => text.into(),
            SyntaxKind::CLASS_KW => text.into(),
            SyntaxKind::CONST_KW => text.into(),
            SyntaxKind::CONTINUE_KW => text.into(),
            SyntaxKind::DEBUGGER_KW => text.into(),
            SyntaxKind::DEFAULT_KW => text.into(),
            SyntaxKind::DELETE_KW => text.into(),
            SyntaxKind::DO_KW => text.into(),
            SyntaxKind::ELSE_KW => text.into(),
            SyntaxKind::ENUM_KW => text.into(),
            SyntaxKind::EXPORT_KW => text.into(),
            SyntaxKind::EXTENDS_KW => text.into(),
            SyntaxKind::FALSE_KW => text.into(),
            SyntaxKind::FINALLY_KW => text.into(),
            SyntaxKind::FOR_KW => text.into(),
            SyntaxKind::FUNCTION_KW => text.into(),
            SyntaxKind::IF_KW => text.into(),
            SyntaxKind::IN_KW => text.into(),
            SyntaxKind::INSTANCEOF_KW => text.into(),
            SyntaxKind::INTERFACE_KW => text.into(),
            SyntaxKind::IMPORT_KW => text.into(),
            SyntaxKind::IMPLEMENTS_KW => text.into(),
            SyntaxKind::NEW_KW => text.into(),
            SyntaxKind::NULL_KW => text.into(),
            SyntaxKind::PACKAGE_KW => text.into(),
            SyntaxKind::PRIVATE_KW => text.into(),
            SyntaxKind::PROTECTED_KW => text.into(),
            SyntaxKind::PUBLIC_KW => text.into(),
            SyntaxKind::RETURN_KW => text.into(),
            SyntaxKind::SUPER_KW => text.into(),
            SyntaxKind::SWITCH_KW => text.into(),
            SyntaxKind::THIS_KW => text.into(),
            SyntaxKind::THROW_KW => text.into(),
            SyntaxKind::TRY_KW => text.into(),
            SyntaxKind::TRUE_KW => text.into(),
            SyntaxKind::TYPEOF_KW => text.into(),
            SyntaxKind::VAR_KW => text.into(),
            SyntaxKind::VOID_KW => text.into(),
            SyntaxKind::WHILE_KW => text.into(),
            SyntaxKind::WITH_KW => text.into(),
            SyntaxKind::YIELD_KW => text.into(),
            SyntaxKind::READONLY_KW => text.into(),
            SyntaxKind::KEYOF_KW => text.into(),
            SyntaxKind::UNIQUE_KW => text.into(),
            SyntaxKind::DECLARE_KW => text.into(),
            SyntaxKind::ABSTRACT_KW => text.into(),
            SyntaxKind::STATIC_KW => text.into(),
            SyntaxKind::ASYNC_KW => text.into(),
            SyntaxKind::TYPE_KW => text.into(),
            SyntaxKind::FROM_KW => text.into(),
            SyntaxKind::AS_KW => text.into(),
            SyntaxKind::REQUIRE_KW => text.into(),
            SyntaxKind::NAMESPACE_KW => text.into(),
            SyntaxKind::ASSERT_KW => text.into(),
            SyntaxKind::MODULE_KW => text.into(),
            SyntaxKind::GLOBAL_KW => text.into(),
            SyntaxKind::INFER_KW => text.into(),
            SyntaxKind::GET_KW => text.into(),
            SyntaxKind::SET_KW => text.into(),
            SyntaxKind::NUMBER => text.into(),
            SyntaxKind::STRING => text.into(),
            SyntaxKind::REGEX => text.into(),
            SyntaxKind::HASH => text.into(),
            SyntaxKind::TEMPLATE_CHUNK => text.into(),
            SyntaxKind::DOLLARCURLY => text.into(),
            SyntaxKind::BACKTICK => text.into(),
            SyntaxKind::ERROR_TOKEN => text.into(),
            SyntaxKind::IDENT => text.into(),
            SyntaxKind::WHITESPACE => text.into(),
            SyntaxKind::COMMENT => text.into(),
            SyntaxKind::SHEBANG => text.into(),
            SyntaxKind::SCRIPT => text.into(),
            SyntaxKind::MODULE => text.into(),
            SyntaxKind::ERROR => text.into(),
            SyntaxKind::BLOCK_STMT => text.into(),
            SyntaxKind::VAR_DECL => text.into(),
            SyntaxKind::DECLARATOR => text.into(),
            SyntaxKind::EMPTY_STMT => text.into(),
            SyntaxKind::EXPR_STMT => text.into(),
            SyntaxKind::IF_STMT => text.into(),
            SyntaxKind::DO_WHILE_STMT => text.into(),
            SyntaxKind::WHILE_STMT => text.into(),
            SyntaxKind::FOR_STMT => text.into(),
            SyntaxKind::FOR_IN_STMT => text.into(),
            SyntaxKind::CONTINUE_STMT => text.into(),
            SyntaxKind::BREAK_STMT => text.into(),
            SyntaxKind::RETURN_STMT => text.into(),
            SyntaxKind::WITH_STMT => text.into(),
            SyntaxKind::SWITCH_STMT => text.into(),
            SyntaxKind::CASE_CLAUSE => text.into(),
            SyntaxKind::DEFAULT_CLAUSE => text.into(),
            SyntaxKind::LABELLED_STMT => text.into(),
            SyntaxKind::THROW_STMT => text.into(),
            SyntaxKind::TRY_STMT => text.into(),
            SyntaxKind::CATCH_CLAUSE => text.into(),
            SyntaxKind::FINALIZER => text.into(),
            SyntaxKind::DEBUGGER_STMT => text.into(),
            SyntaxKind::FN_DECL => text.into(),
            SyntaxKind::NAME => text.into(),
            SyntaxKind::NAME_REF => text.into(),
            SyntaxKind::PARAMETER_LIST => text.into(),
            SyntaxKind::THIS_EXPR => text.into(),
            SyntaxKind::ARRAY_EXPR => text.into(),
            SyntaxKind::OBJECT_EXPR => text.into(),
            SyntaxKind::LITERAL_PROP => text.into(),
            SyntaxKind::GETTER => text.into(),
            SyntaxKind::SETTER => text.into(),
            SyntaxKind::GROUPING_EXPR => text.into(),
            SyntaxKind::NEW_EXPR => text.into(),
            SyntaxKind::FN_EXPR => text.into(),
            SyntaxKind::BRACKET_EXPR => text.into(),
            SyntaxKind::DOT_EXPR => text.into(),
            SyntaxKind::CALL_EXPR => text.into(),
            SyntaxKind::UNARY_EXPR => text.into(),
            SyntaxKind::BIN_EXPR => text.into(),
            SyntaxKind::COND_EXPR => text.into(),
            SyntaxKind::ASSIGN_EXPR => text.into(),
            SyntaxKind::SEQUENCE_EXPR => text.into(),
            SyntaxKind::ARG_LIST => text.into(),
            SyntaxKind::LITERAL => text.into(),
            SyntaxKind::TEMPLATE => text.into(),
            SyntaxKind::TEMPLATE_ELEMENT => text.into(),
            SyntaxKind::CONDITION => text.into(),
            SyntaxKind::SPREAD_ELEMENT => text.into(),
            SyntaxKind::SUPER_CALL => text.into(),
            SyntaxKind::IMPORT_CALL => text.into(),
            SyntaxKind::NEW_TARGET => text.into(),
            SyntaxKind::IMPORT_META => text.into(),
            SyntaxKind::IDENT_PROP => text.into(),
            SyntaxKind::SPREAD_PROP => text.into(),
            SyntaxKind::INITIALIZED_PROP => text.into(),
            SyntaxKind::OBJECT_PATTERN => text.into(),
            SyntaxKind::ARRAY_PATTERN => text.into(),
            SyntaxKind::ASSIGN_PATTERN => text.into(),
            SyntaxKind::REST_PATTERN => text.into(),
            SyntaxKind::KEY_VALUE_PATTERN => text.into(),
            SyntaxKind::COMPUTED_PROPERTY_NAME => text.into(),
            SyntaxKind::FOR_OF_STMT => text.into(),
            SyntaxKind::SINGLE_PATTERN => text.into(),
            SyntaxKind::ARROW_EXPR => text.into(),
            SyntaxKind::YIELD_EXPR => text.into(),
            SyntaxKind::CLASS_DECL => text.into(),
            SyntaxKind::CLASS_EXPR => text.into(),
            SyntaxKind::CLASS_BODY => text.into(),
            SyntaxKind::METHOD => text.into(),
            SyntaxKind::IMPORT_DECL => text.into(),
            SyntaxKind::EXPORT_DECL => text.into(),
            SyntaxKind::EXPORT_NAMED => text.into(),
            SyntaxKind::EXPORT_DEFAULT_DECL => text.into(),
            SyntaxKind::EXPORT_DEFAULT_EXPR => text.into(),
            SyntaxKind::EXPORT_WILDCARD => text.into(),
            SyntaxKind::WILDCARD_IMPORT => text.into(),
            SyntaxKind::NAMED_IMPORTS => text.into(),
            SyntaxKind::SPECIFIER => text.into(),
            SyntaxKind::AWAIT_EXPR => text.into(),
            SyntaxKind::FOR_STMT_TEST => text.into(),
            SyntaxKind::FOR_STMT_UPDATE => text.into(),
            SyntaxKind::FOR_STMT_INIT => text.into(),
            SyntaxKind::PRIVATE_NAME => text.into(),
            SyntaxKind::CLASS_PROP => text.into(),
            SyntaxKind::PRIVATE_PROP => text.into(),
            SyntaxKind::CONSTRUCTOR => text.into(),
            SyntaxKind::CONSTRUCTOR_PARAMETERS => text.into(),
            SyntaxKind::PRIVATE_PROP_ACCESS => text.into(),
            SyntaxKind::IMPORT_STRING_SPECIFIER => text.into(),
            SyntaxKind::EXPR_PATTERN => text.into(),
            SyntaxKind::TS_ANY => text.into(),
            SyntaxKind::TS_UNKNOWN => text.into(),
            SyntaxKind::TS_NUMBER => text.into(),
            SyntaxKind::TS_OBJECT => text.into(),
            SyntaxKind::TS_BOOLEAN => text.into(),
            SyntaxKind::TS_BIGINT => text.into(),
            SyntaxKind::TS_STRING => text.into(),
            SyntaxKind::TS_SYMBOL => text.into(),
            SyntaxKind::TS_VOID => text.into(),
            SyntaxKind::TS_UNDEFINED => text.into(),
            SyntaxKind::TS_NULL => text.into(),
            SyntaxKind::TS_NEVER => text.into(),
            SyntaxKind::TS_THIS => text.into(),
            SyntaxKind::TS_LITERAL => text.into(),
            SyntaxKind::TS_PREDICATE => text.into(),
            SyntaxKind::TS_TUPLE => text.into(),
            SyntaxKind::TS_TUPLE_ELEMENT => text.into(),
            SyntaxKind::TS_PAREN => text.into(),
            SyntaxKind::TS_TYPE_REF => text.into(),
            SyntaxKind::TS_QUALIFIED_PATH => text.into(),
            SyntaxKind::TS_TYPE_NAME => text.into(),
            SyntaxKind::TS_TEMPLATE => text.into(),
            SyntaxKind::TS_TEMPLATE_ELEMENT => text.into(),
            SyntaxKind::TS_MAPPED_TYPE => text.into(),
            SyntaxKind::TS_MAPPED_TYPE_PARAM => text.into(),
            SyntaxKind::TS_MAPPED_TYPE_READONLY => text.into(),
            SyntaxKind::TS_TYPE_QUERY => text.into(),
            SyntaxKind::TS_TYPE_QUERY_EXPR => text.into(),
            SyntaxKind::TS_IMPORT => text.into(),
            SyntaxKind::TS_TYPE_ARGS => text.into(),
            SyntaxKind::TS_ARRAY => text.into(),
            SyntaxKind::TS_INDEXED_ARRAY => text.into(),
            SyntaxKind::TS_TYPE_OPERATOR => text.into(),
            SyntaxKind::TS_INTERSECTION => text.into(),
            SyntaxKind::TS_UNION => text.into(),
            SyntaxKind::TS_TYPE_PARAMS => text.into(),
            SyntaxKind::TS_FN_TYPE => text.into(),
            SyntaxKind::TS_CONSTRUCTOR_TYPE => text.into(),
            SyntaxKind::TS_EXTENDS => text.into(),
            SyntaxKind::TS_CONDITIONAL_TYPE => text.into(),
            SyntaxKind::TS_CONSTRAINT => text.into(),
            SyntaxKind::TS_DEFAULT => text.into(),
            SyntaxKind::TS_TYPE_PARAM => text.into(),
            SyntaxKind::TS_NON_NULL => text.into(),
            SyntaxKind::TS_ASSERTION => text.into(),
            SyntaxKind::TS_CONST_ASSERTION => text.into(),
            SyntaxKind::TS_ENUM => text.into(),
            SyntaxKind::TS_ENUM_MEMBER => text.into(),
            SyntaxKind::TS_TYPE_ALIAS_DECL => text.into(),
            SyntaxKind::TS_NAMESPACE_DECL => text.into(),
            SyntaxKind::TS_MODULE_BLOCK => text.into(),
            SyntaxKind::TS_MODULE_DECL => text.into(),
            SyntaxKind::TS_CONSTRUCTOR_PARAM => text.into(),
            SyntaxKind::TS_CALL_SIGNATURE_DECL => text.into(),
            SyntaxKind::TS_CONSTRUCT_SIGNATURE_DECL => text.into(),
            SyntaxKind::TS_INDEX_SIGNATURE => text.into(),
            SyntaxKind::TS_METHOD_SIGNATURE => text.into(),
            SyntaxKind::TS_PROPERTY_SIGNATURE => text.into(),
            SyntaxKind::TS_INTERFACE_DECL => text.into(),
            SyntaxKind::TS_ACCESSIBILITY => text.into(),
            SyntaxKind::TS_OBJECT_TYPE => text.into(),
            SyntaxKind::TS_EXPR_WITH_TYPE_ARGS => text.into(),
            SyntaxKind::TS_IMPORT_EQUALS_DECL => text.into(),
            SyntaxKind::TS_MODULE_REF => text.into(),
            SyntaxKind::TS_EXTERNAL_MODULE_REF => text.into(),
            SyntaxKind::TS_EXPORT_ASSIGNMENT => text.into(),
            SyntaxKind::TS_NAMESPACE_EXPORT_DECL => text.into(),
            SyntaxKind::TS_DECORATOR => text.into(),
            SyntaxKind::TS_INFER => text.into(),
            SyntaxKind::__LAST => text.into(),
        }
    }

    fn parse(literal: &str, parent: Option<Rc<RefCell<InMemoryNode<Self>>>>) -> Rc<RefCell<InMemoryNode<Self>>> {
        let parse = parse_text(literal, 0);
        // The untyped syntax node of `foo.bar[2]`, the root node is `Script`.
        let untyped_expr_node = parse.syntax();

        let root = convert_rslint_syntaxnode_to_inmemorynode(untyped_expr_node);

        // Set the parent on the newly parsed node
        root.borrow_mut().parent = if let Some(parent) = parent {
            Some(Rc::downgrade(&parent))
        } else {
            None
        };

        root
    }
}

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

    println!("------ ONE ------");
    let parent = InMemoryNode::<SyntaxKind>::new_tree_from_literal_in_chunks("foo:bar baz hello world", 4);
    InMemoryNode::dump(&parent);

    println!("------");
    InMemoryNode::insert_child(&parent, InMemoryNode::new_from_literal("NEW!"), 4);
    InMemoryNode::dump(&parent);
    println!("------");
    InMemoryNode::insert_child(&parent.borrow().children[2].clone(), InMemoryNode::new_from_literal("BLEW!"), 0);
    InMemoryNode::insert_child(&parent.borrow().children[2].clone(), InMemoryNode::new_from_literal("YOO"), 0);
    InMemoryNode::dump(&parent);
    println!("------");

    let cur = Cursor::new_at(parent.borrow().children[2].clone(), 0);
    let mut selection = cur.selection();
    // selection.set_primary(selection.primary.seek_forwards(CursorSeek::AdvanceByCharCount(2)));
    selection.set_primary(selection.primary.seek_forwards(CursorSeek::advance_lower_word(Inclusivity::Inclusive)));
    // selection.set_primary(selection.primary.seek_forwards(CursorSeek::advance_lower_word(Inclusivity::Exclusive)));

    println!("SELECTION: {selection:?}");
    println!("------ END ONE ------");

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
    let root = InMemoryNode::<SyntaxKind>::new_from_parsed(r#"
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
    "#);
    // let root = InMemoryNode::<SyntaxKind>::new_from_parsed("console.log(123);");
    InMemoryNode::dump(&root);

    let mut selection = Cursor::new(root).selection();
    selection.set_secondary(selection.secondary.seek_forwards_until(|_n, _ct| CursorSeek::Continue));
    println!("RESULT: {:?}", selection);

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
