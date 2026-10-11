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

/// Get the command and arguments from a call that uses the default autovar variable.
///
/// Returns `None` for other expressions, commands without autovar, or calls that
/// supply their own result variable. Does not run the command or check its input arguments.
pub fn autovar_call_parts<'a>(
    expr: &'a Expression,
    db: &DatabaseV2,
) -> Option<(&'a Expression, &'a [Expression])> {
    let ExpressionKind::Call { function, args } = &expr.node else {
        return None;
    };
    let ExpressionKind::Identifier(name) = &function.node else {
        return None;
    };
    let result_index = autovar_param_index(db.get_command(name).ok()?)?;
    (args.len() <= result_index).then_some((function.as_ref(), args.as_slice()))
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
                autovar_call_parts(&expr, db).is_some(),
                expected,
                "{source}"
            );
        }
    }
}
