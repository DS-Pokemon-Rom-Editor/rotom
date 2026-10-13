use crate::compiler::ast::{Expression, ExpressionKind};
use crate::database::{Command, DatabaseV2, ParamDef};

pub const VAR_RESULT: i32 = 0x800C;

const AUTOVAR_DEFAULT_VALUES: &[&str] = &["VAR_RESULT", "0x800C", "VAR_SPECIAL_RESULT"];

pub fn is_autovar_param(param: &ParamDef) -> bool {
    // This intentionally treats only VAR_RESULT-style defaults as autovar.
    // It relies on the database invariant that no non-autovar result default
    // appears outside the final/defaultable result slot.
    param.default.as_ref().is_some_and(|default| {
        AUTOVAR_DEFAULT_VALUES
            .iter()
            .any(|v| default.eq_ignore_ascii_case(v))
    })
}

pub fn autovar_param_index(cmd: &Command) -> Option<usize> {
    cmd.params.iter().position(is_autovar_param)
}

/// Returns the command, arguments, and result variable for the autovar call in `expr`.
///
/// Returns `None` for other expressions, commands without autovar, or calls that
/// put the result variable in their argument list. `resolve` looks up a `.into(...)`
/// destination; without that suffix, the call uses `VAR_RESULT`.
/// Uses `db` to find eligible commands. Does not run commands.
pub fn autovar_call_parts<'a>(
    expr: &'a Expression,
    db: &DatabaseV2,
    resolve: impl Fn(&Expression) -> Option<i32>,
) -> Option<(&'a Expression, &'a [Expression], i32)> {
    let ExpressionKind::Call {
        function,
        args,
        destination,
    } = &expr.node
    else {
        return None;
    };
    let ExpressionKind::Identifier(name) = &function.node else {
        return None;
    };
    let result_index = autovar_param_index(db.get_command(name).ok()?)?;
    let result_var = match destination {
        Some(variable) => resolve(variable)?,
        None => VAR_RESULT,
    };
    (args.len() <= result_index && (0x4000..=0xffff).contains(&result_var)).then_some((
        function.as_ref(),
        args.as_slice(),
        result_var,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::ast::Precedence;
    use crate::compiler::{Lexer, Parser};
    use crate::database::{CommandType, ParamType, Variant};

    fn make_param(name: &str, default: Option<&str>) -> ParamDef {
        ParamDef {
            name: name.to_string(),
            param_type: ParamType::Var,
            const_value: None,
            default: default.map(std::string::ToString::to_string),
            optional: false,
        }
    }

    fn make_command(params: Vec<ParamDef>) -> Command {
        Command {
            cmd_type: CommandType::ScriptCmd,
            id: Some(1),
            legacy_name: None,
            description: None,
            params,
            variants: Option::<Vec<Variant>>::None,
            expansion: None,
        }
    }

    #[test]
    fn test_is_autovar_param_true_for_result_defaults() {
        let p1 = make_param("destVarID", Some("VAR_RESULT"));
        let p2 = make_param("successVar", Some("0x800C"));
        assert!(is_autovar_param(&p1));
        assert!(is_autovar_param(&p2));
    }

    #[test]
    fn test_is_autovar_param_true_without_name_heuristics() {
        let p = make_param("someOtherParam", Some("VAR_RESULT"));
        assert!(is_autovar_param(&p));
    }

    #[test]
    fn test_autovar_param_index_returns_expected_slot() {
        let cmd = make_command(vec![
            make_param("item", None),
            make_param("amount", None),
            make_param("destVarID", Some("VAR_RESULT")),
        ]);
        assert_eq!(autovar_param_index(&cmd), Some(2));
    }

    /// Checks that only default autovar calls are accepted.
    #[test]
    fn test_autovar_call_parts_only_accepts_default_autovar_calls() {
        let db = DatabaseV2::test_platinum();
        for (source, expected) in [
            ("GetPlayerStarterSpecies()", true),
            ("GetPlayerStarterSpecies(0x8000)", false),
            ("PlayCry(0)", false),
            ("UnknownCommand()", false),
            ("GetPlayerStarterSpecies", false),
            ("GetPlayerStarterSpecies() + 1", false),
            ("(1)()", false),
        ] {
            let mut parser = Parser::new(Lexer::new(source));
            let expr = parser.parse_expression(Precedence::Lowest).unwrap();
            assert_eq!(
                autovar_call_parts(&expr, db, |_| None).is_some(),
                expected,
                "{source}"
            );
        }
    }

    #[test]
    fn test_autovar_call_parts_selects_result_variable() {
        for (source, expected) in [
            ("GetPlayerStarterSpecies()", Some(VAR_RESULT)),
            ("GetPlayerStarterSpecies().into(0x8004)", Some(0x8004)),
            ("GetPlayerStarterSpecies().into(RESULT)", Some(0x8005)),
            ("GetPlayerStarterSpecies().into(MISSING)", None),
            ("GetPlayerStarterSpecies().into(1)", None),
            ("GetPlayerStarterSpecies().into(0x10000)", None),
            ("GetPlayerStarterSpecies(0x8004).into(0x8005)", None),
            ("PlayCry(0).into(0x8004)", None),
        ] {
            let mut parser = Parser::new(Lexer::new(source));
            let expr = parser.parse_expression(Precedence::Lowest).unwrap();
            let parts =
                autovar_call_parts(
                    &expr,
                    DatabaseV2::test_platinum(),
                    |variable| match &variable.node {
                        ExpressionKind::Number(value) => Some(*value),
                        ExpressionKind::Identifier(name) if name == "RESULT" => Some(0x8005),
                        _ => None,
                    },
                );
            assert_eq!(parts.map(|(_, _, variable)| variable), expected, "{source}");
        }
    }
}
