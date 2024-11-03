use std::{rc::Rc, cell::RefCell};

use rslint_parser::{parse_text, SyntaxNode, WalkEvent, NodeOrToken, SyntaxKind as SyntaxKindGlobal};
use colored::{ColoredString, Colorize};

use crate::node_tree::node::{TokenKindTrait, InMemoryNode, NodeMetadata};

/// rslint_parser::SyntaxKind is an enum from rslint_parser (javascript parser) which contains all
/// output tokens. It has been re-exported from rslint_parser::SyntaxKind here.
///
/// TokenKindTrait has been implemented on this enum so that this SyntaxKind enum can be used as
/// a generic parameter to InMemoryNode (which allows InMemoryNode to store nodes of this type)
pub type SyntaxKind = SyntaxKindGlobal;

pub fn convert_rslint_syntaxnode_to_inmemorynode(syntax_node: SyntaxNode) -> Rc<RefCell<InMemoryNode<SyntaxKind>>> {
    // let node_literal = match syntax_node.kind() {
    //     rslint_parser::SyntaxKind::TOMBSTONE => format!("TOMBSTONE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EOF => format!("EOF {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SEMICOLON => format!("SEMICOLON {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::COMMA => format!("COMMA {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::L_PAREN => format!("L_PAREN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::R_PAREN => format!("R_PAREN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::L_CURLY => format!("L_CURLY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::R_CURLY => format!("R_CURLY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::L_BRACK => format!("L_BRACK {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::R_BRACK => format!("R_BRACK {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::L_ANGLE => format!("L_ANGLE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::R_ANGLE => format!("R_ANGLE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TILDE => format!("TILDE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::QUESTION => format!("QUESTION {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::QUESTION2 => format!("QUESTION2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::QUESTIONDOT => format!("QUESTIONDOT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AMP => format!("AMP {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PIPE => format!("PIPE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PLUS => format!("PLUS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PLUS2 => format!("PLUS2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::STAR => format!("STAR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::STAR2 => format!("STAR2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SLASH => format!("SLASH {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CARET => format!("CARET {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PERCENT => format!("PERCENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DOT => format!("DOT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DOT2 => format!("DOT2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::COLON => format!("COLON {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EQ => format!("EQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EQ2 => format!("EQ2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EQ3 => format!("EQ3 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FAT_ARROW => format!("FAT_ARROW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::BANG => format!("BANG {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NEQ => format!("NEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NEQ2 => format!("NEQ2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::MINUS => format!("MINUS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::MINUS2 => format!("MINUS2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::LTEQ => format!("LTEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::GTEQ => format!("GTEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PLUSEQ => format!("PLUSEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::MINUSEQ => format!("MINUSEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PIPEEQ => format!("PIPEEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AMPEQ => format!("AMPEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CARETEQ => format!("CARETEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SLASHEQ => format!("SLASHEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::STAREQ => format!("STAREQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PERCENTEQ => format!("PERCENTEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AMP2 => format!("AMP2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PIPE2 => format!("PIPE2 {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SHL => format!("SHL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SHR => format!("SHR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::USHR => format!("USHR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SHLEQ => format!("SHLEQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SHREQ => format!("SHREQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::USHREQ => format!("USHREQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AMP2EQ => format!("AMP2EQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PIPE2EQ => format!("PIPE2EQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::STAR2EQ => format!("STAR2EQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::QUESTION2EQ => format!("QUESTION2EQ {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AT => format!("AT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AWAIT_KW => format!("AWAIT_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::BREAK_KW => format!("BREAK_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CASE_KW => format!("CASE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CATCH_KW => format!("CATCH_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CLASS_KW => format!("CLASS_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CONST_KW => format!("CONST_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CONTINUE_KW => format!("CONTINUE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DEBUGGER_KW => format!("DEBUGGER_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DEFAULT_KW => format!("DEFAULT_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DELETE_KW => format!("DELETE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DO_KW => format!("DO_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ELSE_KW => format!("ELSE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ENUM_KW => format!("ENUM_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPORT_KW => format!("EXPORT_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXTENDS_KW => format!("EXTENDS_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FALSE_KW => format!("FALSE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FINALLY_KW => format!("FINALLY_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FOR_KW => format!("FOR_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FUNCTION_KW => format!("FUNCTION_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IF_KW => format!("IF_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IN_KW => format!("IN_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::INSTANCEOF_KW => format!("INSTANCEOF_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::INTERFACE_KW => format!("INTERFACE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IMPORT_KW => format!("IMPORT_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IMPLEMENTS_KW => format!("IMPLEMENTS_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NEW_KW => format!("NEW_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NULL_KW => format!("NULL_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PACKAGE_KW => format!("PACKAGE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PRIVATE_KW => format!("PRIVATE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PROTECTED_KW => format!("PROTECTED_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PUBLIC_KW => format!("PUBLIC_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::RETURN_KW => format!("RETURN_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SUPER_KW => format!("SUPER_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SWITCH_KW => format!("SWITCH_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::THIS_KW => format!("THIS_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::THROW_KW => format!("THROW_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TRY_KW => format!("TRY_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TRUE_KW => format!("TRUE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TYPEOF_KW => format!("TYPEOF_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::VAR_KW => format!("VAR_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::VOID_KW => format!("VOID_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::WHILE_KW => format!("WHILE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::WITH_KW => format!("WITH_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::YIELD_KW => format!("YIELD_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::READONLY_KW => format!("READONLY_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::KEYOF_KW => format!("KEYOF_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::UNIQUE_KW => format!("UNIQUE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DECLARE_KW => format!("DECLARE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ABSTRACT_KW => format!("ABSTRACT_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::STATIC_KW => format!("STATIC_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ASYNC_KW => format!("ASYNC_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TYPE_KW => format!("TYPE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FROM_KW => format!("FROM_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AS_KW => format!("AS_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::REQUIRE_KW => format!("REQUIRE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NAMESPACE_KW => format!("NAMESPACE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ASSERT_KW => format!("ASSERT_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::MODULE_KW => format!("MODULE_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::GLOBAL_KW => format!("GLOBAL_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::INFER_KW => format!("INFER_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::GET_KW => format!("GET_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SET_KW => format!("SET_KW {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NUMBER => format!("NUMBER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::STRING => format!("STRING {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::REGEX => format!("REGEX {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::HASH => format!("HASH {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TEMPLATE_CHUNK => format!("TEMPLATE_CHUNK {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DOLLARCURLY => format!("DOLLARCURLY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::BACKTICK => format!("BACKTICK {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ERROR_TOKEN => format!("ERROR_TOKEN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IDENT => format!("IDENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::WHITESPACE => format!("WHITESPACE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::COMMENT => format!("COMMENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SHEBANG => format!("SHEBANG {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SCRIPT => format!("SCRIPT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::MODULE => format!("MODULE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ERROR => format!("ERROR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::BLOCK_STMT => format!("BLOCK_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::VAR_DECL => format!("VAR_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DECLARATOR => format!("DECLARATOR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EMPTY_STMT => format!("EMPTY_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPR_STMT => format!("EXPR_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IF_STMT => format!("IF_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DO_WHILE_STMT => format!("DO_WHILE_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::WHILE_STMT => format!("WHILE_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FOR_STMT => format!("FOR_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FOR_IN_STMT => format!("FOR_IN_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CONTINUE_STMT => format!("CONTINUE_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::BREAK_STMT => format!("BREAK_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::RETURN_STMT => format!("RETURN_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::WITH_STMT => format!("WITH_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SWITCH_STMT => format!("SWITCH_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CASE_CLAUSE => format!("CASE_CLAUSE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DEFAULT_CLAUSE => format!("DEFAULT_CLAUSE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::LABELLED_STMT => format!("LABELLED_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::THROW_STMT => format!("THROW_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TRY_STMT => format!("TRY_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CATCH_CLAUSE => format!("CATCH_CLAUSE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FINALIZER => format!("FINALIZER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DEBUGGER_STMT => format!("DEBUGGER_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FN_DECL => format!("FN_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NAME => format!("NAME {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NAME_REF => format!("NAME_REF {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PARAMETER_LIST => format!("PARAMETER_LIST {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::THIS_EXPR => format!("THIS_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ARRAY_EXPR => format!("ARRAY_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::OBJECT_EXPR => format!("OBJECT_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::LITERAL_PROP => format!("LITERAL_PROP {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::GETTER => format!("GETTER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SETTER => format!("SETTER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::GROUPING_EXPR => format!("GROUPING_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NEW_EXPR => format!("NEW_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FN_EXPR => format!("FN_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::BRACKET_EXPR => format!("BRACKET_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::DOT_EXPR => format!("DOT_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CALL_EXPR => format!("CALL_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::UNARY_EXPR => format!("UNARY_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::BIN_EXPR => format!("BIN_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::COND_EXPR => format!("COND_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ASSIGN_EXPR => format!("ASSIGN_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SEQUENCE_EXPR => format!("SEQUENCE_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ARG_LIST => format!("ARG_LIST {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::LITERAL => format!("LITERAL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TEMPLATE => format!("TEMPLATE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TEMPLATE_ELEMENT => format!("TEMPLATE_ELEMENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CONDITION => format!("CONDITION {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SPREAD_ELEMENT => format!("SPREAD_ELEMENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SUPER_CALL => format!("SUPER_CALL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IMPORT_CALL => format!("IMPORT_CALL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NEW_TARGET => format!("NEW_TARGET {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IMPORT_META => format!("IMPORT_META {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IDENT_PROP => format!("IDENT_PROP {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SPREAD_PROP => format!("SPREAD_PROP {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::INITIALIZED_PROP => format!("INITIALIZED_PROP {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::OBJECT_PATTERN => format!("OBJECT_PATTERN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ARRAY_PATTERN => format!("ARRAY_PATTERN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ASSIGN_PATTERN => format!("ASSIGN_PATTERN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::REST_PATTERN => format!("REST_PATTERN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::KEY_VALUE_PATTERN => format!("KEY_VALUE_PATTERN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::COMPUTED_PROPERTY_NAME => format!("COMPUTED_PROPERTY_NAME {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FOR_OF_STMT => format!("FOR_OF_STMT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SINGLE_PATTERN => format!("SINGLE_PATTERN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::ARROW_EXPR => format!("ARROW_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::YIELD_EXPR => format!("YIELD_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CLASS_DECL => format!("CLASS_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CLASS_EXPR => format!("CLASS_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CLASS_BODY => format!("CLASS_BODY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::METHOD => format!("METHOD {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IMPORT_DECL => format!("IMPORT_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPORT_DECL => format!("EXPORT_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPORT_NAMED => format!("EXPORT_NAMED {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPORT_DEFAULT_DECL => format!("EXPORT_DEFAULT_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPORT_DEFAULT_EXPR => format!("EXPORT_DEFAULT_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPORT_WILDCARD => format!("EXPORT_WILDCARD {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::WILDCARD_IMPORT => format!("WILDCARD_IMPORT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::NAMED_IMPORTS => format!("NAMED_IMPORTS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::SPECIFIER => format!("SPECIFIER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::AWAIT_EXPR => format!("AWAIT_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FOR_STMT_TEST => format!("FOR_STMT_TEST {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FOR_STMT_UPDATE => format!("FOR_STMT_UPDATE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::FOR_STMT_INIT => format!("FOR_STMT_INIT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PRIVATE_NAME => format!("PRIVATE_NAME {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CLASS_PROP => format!("CLASS_PROP {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PRIVATE_PROP => format!("PRIVATE_PROP {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CONSTRUCTOR => format!("CONSTRUCTOR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::CONSTRUCTOR_PARAMETERS => format!("CONSTRUCTOR_PARAMETERS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::PRIVATE_PROP_ACCESS => format!("PRIVATE_PROP_ACCESS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::IMPORT_STRING_SPECIFIER => format!("IMPORT_STRING_SPECIFIER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::EXPR_PATTERN => format!("EXPR_PATTERN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_ANY => format!("TS_ANY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_UNKNOWN => format!("TS_UNKNOWN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_NUMBER => format!("TS_NUMBER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_OBJECT => format!("TS_OBJECT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_BOOLEAN => format!("TS_BOOLEAN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_BIGINT => format!("TS_BIGINT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_STRING => format!("TS_STRING {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_SYMBOL => format!("TS_SYMBOL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_VOID => format!("TS_VOID {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_UNDEFINED => format!("TS_UNDEFINED {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_NULL => format!("TS_NULL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_NEVER => format!("TS_NEVER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_THIS => format!("TS_THIS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_LITERAL => format!("TS_LITERAL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_PREDICATE => format!("TS_PREDICATE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TUPLE => format!("TS_TUPLE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TUPLE_ELEMENT => format!("TS_TUPLE_ELEMENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_PAREN => format!("TS_PAREN {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_REF => format!("TS_TYPE_REF {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_QUALIFIED_PATH => format!("TS_QUALIFIED_PATH {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_NAME => format!("TS_TYPE_NAME {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TEMPLATE => format!("TS_TEMPLATE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TEMPLATE_ELEMENT => format!("TS_TEMPLATE_ELEMENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_MAPPED_TYPE => format!("TS_MAPPED_TYPE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_MAPPED_TYPE_PARAM => format!("TS_MAPPED_TYPE_PARAM {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_MAPPED_TYPE_READONLY => format!("TS_MAPPED_TYPE_READONLY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_QUERY => format!("TS_TYPE_QUERY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_QUERY_EXPR => format!("TS_TYPE_QUERY_EXPR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_IMPORT => format!("TS_IMPORT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_ARGS => format!("TS_TYPE_ARGS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_ARRAY => format!("TS_ARRAY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_INDEXED_ARRAY => format!("TS_INDEXED_ARRAY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_OPERATOR => format!("TS_TYPE_OPERATOR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_INTERSECTION => format!("TS_INTERSECTION {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_UNION => format!("TS_UNION {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_PARAMS => format!("TS_TYPE_PARAMS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_FN_TYPE => format!("TS_FN_TYPE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_CONSTRUCTOR_TYPE => format!("TS_CONSTRUCTOR_TYPE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_EXTENDS => format!("TS_EXTENDS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_CONDITIONAL_TYPE => format!("TS_CONDITIONAL_TYPE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_CONSTRAINT => format!("TS_CONSTRAINT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_DEFAULT => format!("TS_DEFAULT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_PARAM => format!("TS_TYPE_PARAM {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_NON_NULL => format!("TS_NON_NULL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_ASSERTION => format!("TS_ASSERTION {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_CONST_ASSERTION => format!("TS_CONST_ASSERTION {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_ENUM => format!("TS_ENUM {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_ENUM_MEMBER => format!("TS_ENUM_MEMBER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_TYPE_ALIAS_DECL => format!("TS_TYPE_ALIAS_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_NAMESPACE_DECL => format!("TS_NAMESPACE_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_MODULE_BLOCK => format!("TS_MODULE_BLOCK {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_MODULE_DECL => format!("TS_MODULE_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_CONSTRUCTOR_PARAM => format!("TS_CONSTRUCTOR_PARAM {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_CALL_SIGNATURE_DECL => format!("TS_CALL_SIGNATURE_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_CONSTRUCT_SIGNATURE_DECL => format!("TS_CONSTRUCT_SIGNATURE_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_INDEX_SIGNATURE => format!("TS_INDEX_SIGNATURE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_METHOD_SIGNATURE => format!("TS_METHOD_SIGNATURE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_PROPERTY_SIGNATURE => format!("TS_PROPERTY_SIGNATURE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_INTERFACE_DECL => format!("TS_INTERFACE_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_ACCESSIBILITY => format!("TS_ACCESSIBILITY {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_OBJECT_TYPE => format!("TS_OBJECT_TYPE {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_EXPR_WITH_TYPE_ARGS => format!("TS_EXPR_WITH_TYPE_ARGS {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_IMPORT_EQUALS_DECL => format!("TS_IMPORT_EQUALS_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_MODULE_REF => format!("TS_MODULE_REF {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_EXTERNAL_MODULE_REF => format!("TS_EXTERNAL_MODULE_REF {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_EXPORT_ASSIGNMENT => format!("TS_EXPORT_ASSIGNMENT {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_NAMESPACE_EXPORT_DECL => format!("TS_NAMESPACE_EXPORT_DECL {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_DECORATOR => format!("TS_DECORATOR {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::TS_INFER => format!("TS_INFER {:?}", syntax_node.text()),
    //     rslint_parser::SyntaxKind::__LAST => format!("__LAST {:?}", syntax_node.text()),
    // };
    // let node = InMemoryNode::new_from_literal(&node_literal);

    // for child in syntax_node.children() {
    //     InMemoryNode::append_child(&node, convert_rslint_syntaxnode_to_inmemorynode(child));
    // }
    let root = InMemoryNode::new_empty();
    let mut pointer = root.clone();
    let mut level = 0;
    for event in syntax_node.preorder_with_tokens() {
        match event {
            WalkEvent::Enter(element) => {
                // for _ in 0..level {
                //     write!(f, "  ")?;
                // }
                let node_metadata = match element {
                    NodeOrToken::Node(node) => NodeMetadata::AstNode {
                        kind: node.kind(),
                        literal: Some(format!("{}", node.text())),
                    },
                    NodeOrToken::Token(token) => NodeMetadata::AstNode {
                        kind: token.kind(),
                        literal: Some(format!("{}", token.text())),
                    },
                };
                let child = InMemoryNode::new_with_metadata(node_metadata);
                let child_literal = InMemoryNode::literal(&child);

                // Remove literal text from parent nodes that is replicated in the child node
                let parent_metadata = pointer.borrow().metadata.clone();
                if let NodeMetadata::AstNode{ kind, literal: Some(pointer_literal) } = parent_metadata {
                    let new_literal = if pointer_literal.starts_with(&child_literal) {
                        pointer_literal.chars().skip(child_literal.len()).collect::<String>()
                    } else {
                        pointer_literal
                    };
                    pointer.borrow_mut().metadata = NodeMetadata::AstNode {
                        kind,
                        literal: if new_literal.is_empty() { None } else { Some(new_literal) },
                    };
                }

                pointer = InMemoryNode::append_child(&pointer, child);
                level += 1;
            }
            WalkEvent::Leave(_) => {
                let parent_upgraded = pointer.borrow().parent.as_ref().map(|n| n.upgrade());
                if let Some(Some(parent)) = parent_upgraded {
                    pointer = parent;
                }

                level -= 1;
            },
        }
    }
    assert_eq!(level, 0);

    // An optimization: if the generated token tree's root only has a single node within it (should
    // always be the case, just a SCRIPT node), then return that and ditch the EMPTY wrapper
    if root.borrow().children.len() == 1 {
        root.borrow().children.first().unwrap().to_owned()
    } else {
        root
    }
}

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
        if parent_kind == SyntaxKind::LITERAL && kind == SyntaxKind::NUMBER {
            return ColoredString::from(text).purple();
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

    fn is_reparsable(&self) -> bool {
        matches!(self, SyntaxKind::SCRIPT | SyntaxKind::EXPR_STMT)
    }

    fn parse(literal: &str, parent: Option<Rc<RefCell<InMemoryNode<Self>>>>) -> Rc<RefCell<InMemoryNode<Self>>> {
        let parse = parse_text(literal, 0);
        // The untyped syntax node of `foo.bar[2]`, the root node is `Script`.
        let untyped_expr_node = parse.syntax();

        let root = convert_rslint_syntaxnode_to_inmemorynode(untyped_expr_node);

        // If the parsed result output has within it only one node, then extract just the node that
        // was previously the parent to minimize the number of output nodes in the resulting tree
        //
        // However, only traverse down while AST nodes have a single child so that any omitted
        // nodes won't change the literal text contents of the result. ie, in the below, only go
        // down as far as C before bailing out early:
        // - A
        //   - B
        //     - C
        //       - D
        //       - E
        //       - F
        if let Some(NodeMetadata::AstNode { kind: parent_kind, .. }) = parent.as_ref().map(|n| n.borrow().metadata.clone()) {
            let mut pointer = root.clone();
            while {
                if pointer.borrow().children.len() != 1 {
                    false
                } else if let NodeMetadata::AstNode { kind, .. } = pointer.borrow().metadata {
                    kind != parent_kind
                } else {
                    false
                }
            } {
                let Some(first_child) = pointer.borrow().first_child.as_ref().map(|n| n.upgrade()).flatten() else {
                    // Traversal downwards is no longer possible, but a matching AstNode kind has
                    // not been found.
                    //
                    // So, bail out of this optimization, and just return root. The newly generated
                    // AST is signifigantly enough different where this path is not viable.
                    return root;
                };
                pointer = first_child;
            }

            pointer
        } else {
            root
        }
    }
}
