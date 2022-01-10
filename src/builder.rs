#[derive(PartialEq)]
pub enum Step {
    Comment(String),
    Dedent,
    Format(rnix::SyntaxElement),
    FormatWider(rnix::SyntaxElement),
    Indent,
    NewLine,
    Pad,
    Token(rnix::SyntaxKind, String),
    Whitespace,
}

#[derive(Clone)]
pub struct BuildCtx {
    pub config:      crate::config::Config,
    pub force_wide:  bool,
    pub indentation: usize,
    pub pos_new:     crate::position::Position,
    pub pos_old:     crate::position::Position,
}

impl BuildCtx {
    pub fn new(
        config: crate::config::Config,
        force_wide: bool,
        pos_new: crate::position::Position,
        pos_old: crate::position::Position,
    ) -> BuildCtx {
        BuildCtx { config, force_wide, indentation: 0, pos_new, pos_old }
    }
}

pub fn build(
    config: &crate::config::Config,
    element: rnix::SyntaxElement,
    force_wide: bool,
    path: &str,
) -> Option<rowan::GreenNode> {
    let mut builder = rowan::GreenNodeBuilder::new();
    let mut build_ctx = BuildCtx::new(
        config.clone(),
        force_wide,
        crate::position::Position::new(),
        crate::position::Position::new(),
    );

    build_step(
        &mut builder,
        &mut build_ctx,
        path,
        &crate::builder::Step::Format(element),
    );

    if build_ctx.force_wide && build_ctx.pos_new.line > 1 {
        None
    } else {
        Some(builder.finish())
    }
}

fn build_step(
    builder: &mut rowan::GreenNodeBuilder,
    build_ctx: &mut BuildCtx,
    path: &str,
    step: &crate::builder::Step,
) {
    if build_ctx.force_wide && build_ctx.pos_new.line > 1 {
        return;
    }

    match step {
        crate::builder::Step::Comment(text) => {
            add_token(
                builder,
                build_ctx,
                rnix::SyntaxKind::TOKEN_COMMENT,
                text,
            );
        }
        crate::builder::Step::Dedent => {
            build_ctx.indentation -= 1;
        }
        crate::builder::Step::Format(element) => {
            format(builder, build_ctx, element, path);
        }
        crate::builder::Step::FormatWider(element) => {
            format_wider(builder, build_ctx, element, path);
        }
        crate::builder::Step::Indent => {
            build_ctx.indentation += 1;
        }
        crate::builder::Step::NewLine => {
            add_token(
                builder,
                build_ctx,
                rnix::SyntaxKind::TOKEN_WHITESPACE,
                "\n",
            );
        }
        crate::builder::Step::Pad => {
            if build_ctx.indentation > 0 {
                add_token(
                    builder,
                    build_ctx,
                    rnix::SyntaxKind::TOKEN_COMMA,
                    &format!("{0:<1$}", "", 2 * build_ctx.indentation),
                );
            }
        }
        crate::builder::Step::Token(kind, text) => {
            add_token(builder, build_ctx, *kind, &text);
        }
        crate::builder::Step::Whitespace => {
            add_token(
                builder,
                build_ctx,
                rnix::SyntaxKind::TOKEN_WHITESPACE,
                " ",
            );
        }
    }
}

fn add_token(
    builder: &mut rowan::GreenNodeBuilder,
    build_ctx: &mut BuildCtx,
    kind: rnix::SyntaxKind,
    text: &str,
) {
    builder.token(rowan::SyntaxKind(kind as u16), text);
    build_ctx.pos_new.update(text);
}

fn format(
    builder: &mut rowan::GreenNodeBuilder,
    build_ctx: &mut BuildCtx,
    element: &rnix::SyntaxElement,
    path: &str,
) {
    let kind = element.kind();

    match element {
        rnix::SyntaxElement::Node(node) => {
            builder.start_node(rowan::SyntaxKind(kind as u16));

            let rule = match kind {
                rnix::SyntaxKind::NODE_APPLY => crate::rules::apply::rule,
                rnix::SyntaxKind::NODE_ASSERT => crate::rules::assert::rule,
                rnix::SyntaxKind::NODE_ATTR_SET => crate::rules::attr_set::rule,
                rnix::SyntaxKind::NODE_BIN_OP => crate::rules::bin_op::rule,
                rnix::SyntaxKind::NODE_DYNAMIC => crate::rules::dynamic::rule,
                rnix::SyntaxKind::NODE_ERROR => {
                    eprintln!("Warning: found an error node at: {}", path);
                    crate::rules::default
                }
                rnix::SyntaxKind::NODE_IDENT => crate::rules::default,
                rnix::SyntaxKind::NODE_IF_ELSE => crate::rules::if_else::rule,
                rnix::SyntaxKind::NODE_INHERIT => crate::rules::inherit::rule,
                rnix::SyntaxKind::NODE_INHERIT_FROM => {
                    crate::rules::inherit_from::rule
                }
                rnix::SyntaxKind::NODE_KEY => crate::rules::default,
                rnix::SyntaxKind::NODE_KEY_VALUE => {
                    crate::rules::key_value::rule
                }
                rnix::SyntaxKind::NODE_LAMBDA => crate::rules::lambda::rule,
                rnix::SyntaxKind::NODE_LET_IN => crate::rules::let_in::rule,
                rnix::SyntaxKind::NODE_LIST => crate::rules::list::rule,
                rnix::SyntaxKind::NODE_LITERAL => crate::rules::default,
                rnix::SyntaxKind::NODE_LEGACY_LET => {
                    eprintln!(
                        "Warning: found a `legacy let` expression at: {}",
                        path
                    );
                    crate::rules::default
                }
                rnix::SyntaxKind::NODE_OR_DEFAULT => {
                    crate::rules::or_default::rule
                }
                rnix::SyntaxKind::NODE_PAREN => crate::rules::paren::rule,
                rnix::SyntaxKind::NODE_PAT_BIND => crate::rules::pat_bind::rule,
                rnix::SyntaxKind::NODE_PATTERN => crate::rules::pattern::rule,
                rnix::SyntaxKind::NODE_PAT_ENTRY => {
                    crate::rules::pat_entry::rule
                }
                rnix::SyntaxKind::NODE_PATH_WITH_INTERPOL => {
                    crate::rules::default
                }
                rnix::SyntaxKind::NODE_ROOT => crate::rules::root::rule,
                rnix::SyntaxKind::NODE_SELECT => crate::rules::select::rule,
                rnix::SyntaxKind::NODE_STRING => crate::rules::default,
                rnix::SyntaxKind::NODE_STRING_INTERPOL => {
                    crate::rules::string_interpol::rule
                }
                rnix::SyntaxKind::NODE_UNARY_OP => crate::rules::default,
                rnix::SyntaxKind::NODE_WITH => crate::rules::with::rule,
                kind => {
                    panic!("Missing rule for {:?} at: {}", kind, path);
                }
            };

            for step in rule(build_ctx, node) {
                build_step(builder, build_ctx, path, &step);
            }

            builder.finish_node();
        }
        rnix::SyntaxElement::Token(token) => {
            add_token(builder, build_ctx, kind, token.text());
            build_ctx.pos_old.update(token.text());
        }
    }
}

fn format_wider(
    builder: &mut rowan::GreenNodeBuilder,
    build_ctx: &mut BuildCtx,
    element: &rnix::SyntaxElement,
    path: &str,
) {
    match element {
        rnix::SyntaxElement::Node(node) => {
            let maybe_green_node = build(
                &build_ctx.config.with_layout(crate::config::Layout::Wide),
                node.clone().into(),
                true,
                path,
            );

            let layout = match maybe_green_node {
                Some(finished) => {
                    if build_ctx.pos_new.column
                        + finished.to_string().chars().count()
                        > build_ctx.config.max_width()
                    {
                        crate::config::Layout::Tall
                    } else {
                        crate::config::Layout::Wide
                    }
                }
                None => crate::config::Layout::Tall,
            };

            let mut build_ctx_clone = build_ctx.clone();
            build_ctx_clone.config = build_ctx.config.with_layout(layout);
            format(builder, &mut build_ctx_clone, element, path);
        }
        rnix::SyntaxElement::Token(_) => {
            format(builder, build_ctx, element, path);
        }
    };
}