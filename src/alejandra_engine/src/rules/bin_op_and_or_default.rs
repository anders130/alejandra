pub fn rule(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
) -> std::collections::LinkedList<crate::builder::Step> {
    rule_with_configuration(build_ctx, node, "bin_op_and_or_default")
}

pub fn rule_with_configuration(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
    parent_kind: &str,
) -> std::collections::LinkedList<crate::builder::Step> {
    let mut steps = std::collections::LinkedList::new();

    let mut children = crate::children::Children::new_with_configuration(
        build_ctx, node, true,
    );

    let layout = if children.has_comments() || children.has_newlines() {
        &crate::config::Layout::Tall
    } else {
        build_ctx.config.layout()
    };

    // expr
    let child = children.get_next().unwrap();
    match layout {
        crate::config::Layout::Tall => {
            let kind = child.element.kind();

            if (parent_kind == "bin_op_and_or_default"
                && matches!(
                    kind,
                    rnix::SyntaxKind::NODE_BIN_OP
                        | rnix::SyntaxKind::NODE_OR_DEFAULT
                ))
                || (parent_kind == "select"
                    && matches!(kind, rnix::SyntaxKind::NODE_SELECT))
            {
                steps.push_back(crate::builder::Step::Format(child.element));
            } else {
                steps.push_back(crate::builder::Step::FormatWider(
                    child.element,
                ));
            }
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
        crate::config::Layout::Wide => {
            steps.push_back(crate::builder::Step::Format(child.element));
        }
    }

    // /**/
    children.drain_comments_and_newlines(|element| match element {
        crate::children::DrainCommentOrNewline::Comment(text) => {
            steps.push_back(crate::builder::Step::Comment(text));
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
        crate::children::DrainCommentOrNewline::Newline(_) => {}
    });

    // operator
    let child = children.get_next().unwrap();
    match layout {
        crate::config::Layout::Tall => {}
        crate::config::Layout::Wide => {
            if parent_kind == "bin_op_and_or_default" {
                steps.push_back(crate::builder::Step::Whitespace);
            }
        }
    }
    steps.push_back(crate::builder::Step::Format(child.element));

    // /**/
    let mut comment = false;
    children.drain_comments_and_newlines(|element| match element {
        crate::children::DrainCommentOrNewline::Comment(text) => {
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
            steps.push_back(crate::builder::Step::Comment(text));
            comment = true;
        }
        crate::children::DrainCommentOrNewline::Newline(_) => {}
    });

    if comment {
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    } else if parent_kind == "bin_op_and_or_default" {
        steps.push_back(crate::builder::Step::Whitespace);
    }

    // expr
    let child = children.get_next().unwrap();
    match layout {
        crate::config::Layout::Tall => {
            steps.push_back(crate::builder::Step::FormatWider(child.element));
        }
        crate::config::Layout::Wide => {
            steps.push_back(crate::builder::Step::Format(child.element));
        }
    }

    steps
}