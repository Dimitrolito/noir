use noirc_frontend::{
    ast::{
        ArrayLiteral, AssignStatement, BlockExpression, CallExpression, CastExpression,
        ConstrainStatement, ConstructorExpression, Expression, ExpressionKind, ForLoopStatement,
        ForRange, FunctionReturnType, Ident, IfExpression, IndexExpression, InfixExpression,
        LValue, Lambda, LetStatement, Literal, MemberAccessExpression, MethodCallExpression,
        ModuleDeclaration, NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, Path,
        PathSegment, Pattern, PrefixExpression, Statement, StatementKind, TraitImplItem, TraitItem,
        TypeImpl, UnresolvedGeneric, UnresolvedGenerics, UnresolvedTraitConstraint, UnresolvedType,
        UnresolvedTypeData, UnresolvedTypeExpression, UseTree, UseTreeKind,
    },
    parser::{Item, ItemKind, ParsedSubModule, ParserError},
    ParsedModule,
};

/// Parses a program and will clear out (set them to a default) any spans in it if `empty_spans` is true.
/// We want to do this in code generated by macros when running in LSP mode so that the generated
/// code doesn't end up overlapping real code, messing with how inlay hints, hover, etc., work.
pub fn parse_program(source_program: &str, empty_spans: bool) -> (ParsedModule, Vec<ParserError>) {
    let (mut parsed_program, errors) = noirc_frontend::parse_program(source_program);
    if empty_spans {
        empty_parsed_module(&mut parsed_program);
    }
    (parsed_program, errors)
}

fn empty_parsed_module(parsed_module: &mut ParsedModule) {
    for item in parsed_module.items.iter_mut() {
        empty_item(item);
    }
}

fn empty_item(item: &mut Item) {
    item.span = Default::default();

    match &mut item.kind {
        ItemKind::Function(noir_function) => empty_noir_function(noir_function),
        ItemKind::Trait(noir_trait) => {
            empty_noir_trait(noir_trait);
        }
        ItemKind::TraitImpl(noir_trait_impl) => {
            empty_noir_trait_impl(noir_trait_impl);
        }
        ItemKind::Impl(type_impl) => {
            empty_type_impl(type_impl);
        }
        ItemKind::Global(let_statement) => empty_let_statement(let_statement),
        ItemKind::Submodules(parsed_submodule) => {
            empty_parsed_submodule(parsed_submodule);
        }
        ItemKind::ModuleDecl(module_declaration) => empty_module_declaration(module_declaration),
        ItemKind::Import(use_tree) => empty_use_tree(use_tree),
        ItemKind::Struct(noir_struct) => empty_noir_struct(noir_struct),
        ItemKind::TypeAlias(noir_type_alias) => empty_noir_type_alias(noir_type_alias),
    }
}

fn empty_noir_trait(noir_trait: &mut NoirTrait) {
    noir_trait.span = Default::default();

    empty_ident(&mut noir_trait.name);
    empty_unresolved_generics(&mut noir_trait.generics);
    empty_unresolved_trait_constraints(&mut noir_trait.where_clause);
    for item in noir_trait.items.iter_mut() {
        empty_trait_item(item);
    }
}

fn empty_noir_trait_impl(noir_trait_impl: &mut NoirTraitImpl) {
    empty_path(&mut noir_trait_impl.trait_name);
    empty_unresolved_generics(&mut noir_trait_impl.impl_generics);
    empty_unresolved_type(&mut noir_trait_impl.object_type);
    empty_unresolved_trait_constraints(&mut noir_trait_impl.where_clause);
    for item in noir_trait_impl.items.iter_mut() {
        empty_trait_impl_item(item);
    }
}

fn empty_type_impl(type_impl: &mut TypeImpl) {
    empty_unresolved_type(&mut type_impl.object_type);
    type_impl.type_span = Default::default();
    empty_unresolved_generics(&mut type_impl.generics);
    empty_unresolved_trait_constraints(&mut type_impl.where_clause);
    for (noir_function, _) in type_impl.methods.iter_mut() {
        empty_noir_function(noir_function);
    }
}

fn empty_noir_function(noir_function: &mut NoirFunction) {
    let def = &mut noir_function.def;

    def.span = Default::default();
    empty_ident(&mut def.name);
    empty_unresolved_generics(&mut def.generics);

    for param in def.parameters.iter_mut() {
        param.span = Default::default();
        empty_unresolved_type(&mut param.typ);
        empty_pattern(&mut param.pattern);
    }

    empty_unresolved_trait_constraints(&mut def.where_clause);
    empty_function_return_type(&mut def.return_type);
    empty_block_expression(&mut def.body);
}

fn empty_trait_item(trait_item: &mut TraitItem) {
    match trait_item {
        TraitItem::Function { name, generics, parameters, return_type, where_clause, body } => {
            empty_ident(name);
            empty_unresolved_generics(generics);
            for (name, typ) in parameters.iter_mut() {
                empty_ident(name);
                empty_unresolved_type(typ);
            }
            empty_function_return_type(return_type);
            for trait_constraint in where_clause.iter_mut() {
                empty_unresolved_trait_constraint(trait_constraint);
            }
            if let Some(body) = body {
                empty_block_expression(body);
            }
        }
        TraitItem::Constant { name, typ, default_value } => {
            empty_ident(name);
            empty_unresolved_type(typ);
            if let Some(default_value) = default_value {
                empty_expression(default_value);
            }
        }
        TraitItem::Type { name } => {
            empty_ident(name);
        }
    }
}

fn empty_trait_impl_item(trait_impl_item: &mut TraitImplItem) {
    match trait_impl_item {
        TraitImplItem::Function(noir_function) => empty_noir_function(noir_function),
        TraitImplItem::Constant(name, typ, default_value) => {
            empty_ident(name);
            empty_unresolved_type(typ);
            empty_expression(default_value);
        }
        TraitImplItem::Type { name, alias } => {
            empty_ident(name);
            empty_unresolved_type(alias);
        }
    }
}

fn empty_let_statement(let_statement: &mut LetStatement) {
    empty_pattern(&mut let_statement.pattern);
    empty_unresolved_type(&mut let_statement.r#type);
    empty_expression(&mut let_statement.expression);
}

fn empty_parsed_submodule(parsed_submodule: &mut ParsedSubModule) {
    empty_ident(&mut parsed_submodule.name);
    empty_parsed_module(&mut parsed_submodule.contents);
}

fn empty_module_declaration(module_declaration: &mut ModuleDeclaration) {
    empty_ident(&mut module_declaration.ident);
}

fn empty_use_tree(use_tree: &mut UseTree) {
    empty_path(&mut use_tree.prefix);

    match &mut use_tree.kind {
        UseTreeKind::Path(name, alias) => {
            empty_ident(name);
            if let Some(alias) = alias {
                empty_ident(alias);
            }
        }
        UseTreeKind::List(use_trees) => {
            for use_tree in use_trees.iter_mut() {
                empty_use_tree(use_tree);
            }
        }
    }
}

fn empty_noir_struct(noir_struct: &mut NoirStruct) {
    noir_struct.span = Default::default();
    empty_ident(&mut noir_struct.name);
    for (name, typ) in noir_struct.fields.iter_mut() {
        empty_ident(name);
        empty_unresolved_type(typ);
    }
    empty_unresolved_generics(&mut noir_struct.generics);
}

fn empty_noir_type_alias(noir_type_alias: &mut NoirTypeAlias) {
    noir_type_alias.span = Default::default();
    empty_ident(&mut noir_type_alias.name);
    empty_unresolved_type(&mut noir_type_alias.typ);
}

fn empty_block_expression(block_expression: &mut BlockExpression) {
    for statement in block_expression.statements.iter_mut() {
        empty_statement(statement);
    }
}

fn empty_statement(statement: &mut Statement) {
    statement.span = Default::default();

    match &mut statement.kind {
        StatementKind::Let(let_statement) => empty_let_statement(let_statement),
        StatementKind::Constrain(constrain_statement) => {
            empty_constrain_statement(constrain_statement)
        }
        StatementKind::Expression(expression) => empty_expression(expression),
        StatementKind::Assign(assign_statement) => empty_assign_statement(assign_statement),
        StatementKind::For(for_loop_statement) => empty_for_loop_statement(for_loop_statement),
        StatementKind::Comptime(statement) => empty_statement(statement),
        StatementKind::Semi(expression) => empty_expression(expression),
        StatementKind::Break | StatementKind::Continue | StatementKind::Error => (),
    }
}

fn empty_constrain_statement(constrain_statement: &mut ConstrainStatement) {
    empty_expression(&mut constrain_statement.0);
    if let Some(expression) = &mut constrain_statement.1 {
        empty_expression(expression);
    }
}

fn empty_expressions(expressions: &mut [Expression]) {
    for expression in expressions.iter_mut() {
        empty_expression(expression);
    }
}

fn empty_expression(expression: &mut Expression) {
    expression.span = Default::default();

    match &mut expression.kind {
        ExpressionKind::Literal(literal) => empty_literal(literal),
        ExpressionKind::Block(block_expression) => empty_block_expression(block_expression),
        ExpressionKind::Prefix(prefix_expression) => empty_prefix_expression(prefix_expression),
        ExpressionKind::Index(index_expression) => empty_index_expression(index_expression),
        ExpressionKind::Call(call_expression) => empty_call_expression(call_expression),
        ExpressionKind::MethodCall(method_call_expression) => {
            empty_method_call_expression(method_call_expression)
        }
        ExpressionKind::Constructor(constructor_expression) => {
            empty_constructor_expression(constructor_expression)
        }
        ExpressionKind::MemberAccess(member_access_expression) => {
            empty_member_access_expression(member_access_expression)
        }
        ExpressionKind::Cast(cast_expression) => empty_cast_expression(cast_expression),
        ExpressionKind::Infix(infix_expression) => empty_infix_expression(infix_expression),
        ExpressionKind::If(if_expression) => empty_if_expression(if_expression),
        ExpressionKind::Variable(path) => empty_path(path),
        ExpressionKind::Tuple(expressions) => {
            empty_expressions(expressions);
        }
        ExpressionKind::Lambda(lambda) => empty_lambda(lambda),
        ExpressionKind::Parenthesized(expression) => empty_expression(expression),
        ExpressionKind::Unquote(expression) => {
            empty_expression(expression);
        }
        ExpressionKind::Comptime(block_expression, _span) => {
            empty_block_expression(block_expression);
        }
        ExpressionKind::Quote(..) | ExpressionKind::Resolved(_) | ExpressionKind::Error => (),
    }
}

fn empty_assign_statement(assign_statement: &mut AssignStatement) {
    empty_lvalue(&mut assign_statement.lvalue);
    empty_expression(&mut assign_statement.expression);
}

fn empty_for_loop_statement(for_loop_statement: &mut ForLoopStatement) {
    for_loop_statement.span = Default::default();
    empty_ident(&mut for_loop_statement.identifier);
    empty_for_range(&mut for_loop_statement.range);
    empty_expression(&mut for_loop_statement.block);
}

fn empty_unresolved_types(unresolved_types: &mut [UnresolvedType]) {
    for unresolved_type in unresolved_types.iter_mut() {
        empty_unresolved_type(unresolved_type);
    }
}

fn empty_unresolved_type(unresolved_type: &mut UnresolvedType) {
    unresolved_type.span = Default::default();

    match &mut unresolved_type.typ {
        UnresolvedTypeData::Array(unresolved_type_expression, unresolved_type) => {
            empty_unresolved_type_expression(unresolved_type_expression);
            empty_unresolved_type(unresolved_type);
        }
        UnresolvedTypeData::Slice(unresolved_type) => empty_unresolved_type(unresolved_type),
        UnresolvedTypeData::Expression(unresolved_type_expression) => {
            empty_unresolved_type_expression(unresolved_type_expression)
        }
        UnresolvedTypeData::FormatString(unresolved_type_expression, unresolved_type) => {
            empty_unresolved_type_expression(unresolved_type_expression);
            empty_unresolved_type(unresolved_type);
        }
        UnresolvedTypeData::Parenthesized(unresolved_type) => {
            empty_unresolved_type(unresolved_type)
        }
        UnresolvedTypeData::Named(path, unresolved_types, _) => {
            empty_path(path);
            empty_unresolved_types(unresolved_types);
        }
        UnresolvedTypeData::TraitAsType(path, unresolved_types) => {
            empty_path(path);
            empty_unresolved_types(unresolved_types);
        }
        UnresolvedTypeData::MutableReference(unresolved_type) => {
            empty_unresolved_type(unresolved_type)
        }
        UnresolvedTypeData::Tuple(unresolved_types) => empty_unresolved_types(unresolved_types),
        UnresolvedTypeData::Function(args, ret, _env) => {
            empty_unresolved_types(args);
            empty_unresolved_type(ret);
        }
        UnresolvedTypeData::FieldElement
        | UnresolvedTypeData::Integer(_, _)
        | UnresolvedTypeData::Bool
        | UnresolvedTypeData::String(_)
        | UnresolvedTypeData::Unit
        | UnresolvedTypeData::Quoted(_)
        | UnresolvedTypeData::Resolved(_)
        | UnresolvedTypeData::Unspecified
        | UnresolvedTypeData::Error => (),
    }
}

fn empty_unresolved_generics(unresolved_generic: &mut UnresolvedGenerics) {
    for generic in unresolved_generic.iter_mut() {
        empty_unresolved_generic(generic);
    }
}

fn empty_unresolved_generic(unresolved_generic: &mut UnresolvedGeneric) {
    match unresolved_generic {
        UnresolvedGeneric::Variable(ident) => empty_ident(ident),
        UnresolvedGeneric::Numeric { ident, typ } => {
            empty_ident(ident);
            empty_unresolved_type(typ);
        }
    }
}

fn empty_pattern(pattern: &mut Pattern) {
    match pattern {
        Pattern::Identifier(ident) => empty_ident(ident),
        Pattern::Mutable(pattern, _span, _) => {
            empty_pattern(pattern);
        }
        Pattern::Tuple(patterns, _) => {
            for pattern in patterns.iter_mut() {
                empty_pattern(pattern);
            }
        }
        Pattern::Struct(path, patterns, _) => {
            empty_path(path);
            for (name, pattern) in patterns.iter_mut() {
                empty_ident(name);
                empty_pattern(pattern);
            }
        }
    }
}

fn empty_unresolved_trait_constraints(
    unresolved_trait_constriants: &mut [UnresolvedTraitConstraint],
) {
    for trait_constraint in unresolved_trait_constriants.iter_mut() {
        empty_unresolved_trait_constraint(trait_constraint);
    }
}

fn empty_unresolved_trait_constraint(unresolved_trait_constraint: &mut UnresolvedTraitConstraint) {
    empty_unresolved_type(&mut unresolved_trait_constraint.typ);
}

fn empty_function_return_type(function_return_type: &mut FunctionReturnType) {
    match function_return_type {
        FunctionReturnType::Ty(unresolved_type) => empty_unresolved_type(unresolved_type),
        FunctionReturnType::Default(_) => (),
    }
}

fn empty_ident(ident: &mut Ident) {
    ident.0.set_span(Default::default());
}

fn empty_path(path: &mut Path) {
    path.span = Default::default();
    for segment in path.segments.iter_mut() {
        empty_path_segment(segment);
    }
}

fn empty_path_segment(segment: &mut PathSegment) {
    segment.span = Default::default();
    empty_ident(&mut segment.ident);
}

fn empty_literal(literal: &mut Literal) {
    match literal {
        Literal::Array(array_literal) => empty_array_literal(array_literal),
        Literal::Slice(array_literal) => empty_array_literal(array_literal),
        Literal::Bool(_)
        | Literal::Integer(_, _)
        | Literal::Str(_)
        | Literal::RawStr(_, _)
        | Literal::FmtStr(_)
        | Literal::Unit => (),
    }
}

fn empty_array_literal(array_literal: &mut ArrayLiteral) {
    match array_literal {
        ArrayLiteral::Standard(expressions) => {
            empty_expressions(expressions);
        }
        ArrayLiteral::Repeated { repeated_element, length } => {
            empty_expression(repeated_element);
            empty_expression(length);
        }
    }
}

fn empty_prefix_expression(prefix_expression: &mut PrefixExpression) {
    empty_expression(&mut prefix_expression.rhs);
}

fn empty_index_expression(index_expression: &mut IndexExpression) {
    empty_expression(&mut index_expression.collection);
    empty_expression(&mut index_expression.index);
}

fn empty_call_expression(call_expression: &mut CallExpression) {
    empty_expression(&mut call_expression.func);
    empty_expressions(&mut call_expression.arguments);
}

fn empty_method_call_expression(method_call_expression: &mut MethodCallExpression) {
    empty_expression(&mut method_call_expression.object);
    empty_ident(&mut method_call_expression.method_name);
    if let Some(generics) = &mut method_call_expression.generics {
        empty_unresolved_types(generics);
    }
    empty_expressions(&mut method_call_expression.arguments);
}

fn empty_constructor_expression(constructor_expression: &mut ConstructorExpression) {
    empty_path(&mut constructor_expression.type_name);
    for (name, expression) in constructor_expression.fields.iter_mut() {
        empty_ident(name);
        empty_expression(expression);
    }
}

fn empty_member_access_expression(member_access_expression: &mut MemberAccessExpression) {
    empty_expression(&mut member_access_expression.lhs);
    empty_ident(&mut member_access_expression.rhs);
}

fn empty_cast_expression(cast_expression: &mut CastExpression) {
    empty_expression(&mut cast_expression.lhs);
    empty_unresolved_type(&mut cast_expression.r#type);
}

fn empty_infix_expression(infix_expression: &mut InfixExpression) {
    empty_expression(&mut infix_expression.lhs);
    empty_expression(&mut infix_expression.rhs);
}

fn empty_if_expression(if_expression: &mut IfExpression) {
    empty_expression(&mut if_expression.condition);
    empty_expression(&mut if_expression.consequence);
    if let Some(alternative) = &mut if_expression.alternative {
        empty_expression(alternative);
    }
}

fn empty_lambda(lambda: &mut Lambda) {
    for (name, typ) in lambda.parameters.iter_mut() {
        empty_pattern(name);
        empty_unresolved_type(typ);
    }
    empty_unresolved_type(&mut lambda.return_type);
    empty_expression(&mut lambda.body);
}

fn empty_lvalue(lvalue: &mut LValue) {
    match lvalue {
        LValue::Ident(ident) => empty_ident(ident),
        LValue::MemberAccess { ref mut object, ref mut field_name, span: _ } => {
            empty_lvalue(object);
            empty_ident(field_name);
        }
        LValue::Index { ref mut array, ref mut index, span: _ } => {
            empty_lvalue(array);
            empty_expression(index);
        }
        LValue::Dereference(lvalue, _) => empty_lvalue(lvalue),
    }
}

fn empty_for_range(for_range: &mut ForRange) {
    match for_range {
        ForRange::Range(from, to) => {
            empty_expression(from);
            empty_expression(to);
        }
        ForRange::Array(expression) => empty_expression(expression),
    }
}

fn empty_unresolved_type_expression(unresolved_type_expression: &mut UnresolvedTypeExpression) {
    match unresolved_type_expression {
        UnresolvedTypeExpression::Variable(path) => empty_path(path),
        UnresolvedTypeExpression::BinaryOperation(lhs, _, rhs, _) => {
            empty_unresolved_type_expression(lhs);
            empty_unresolved_type_expression(rhs);
        }
        UnresolvedTypeExpression::Constant(_, _) => (),
    }
}
