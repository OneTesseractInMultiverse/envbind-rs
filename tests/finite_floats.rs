#![allow(missing_docs)]

use envbind::{
    BindError, Binder, BindingExt, FloatVar, ListVar, MapEnvironment, ValidationError, validators,
};

const NON_FINITE: [&str; 5] = ["NaN", "inf", "-inf", "1e999", "-1e999"];
const NON_FINITE_VALUES: [f64; 3] = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
const FINITE: [(&str, f64); 8] = [
    ("0", 0.0),
    ("-0", -0.0),
    ("1e3", 1000.0),
    ("-2.5e-2", -0.025),
    ("5e-324", f64::from_bits(1)),
    ("-1e-999", -0.0),
    ("1.7976931348623157e308", f64::MAX),
    ("-1.7976931348623157e308", f64::MIN),
];

fn binder(raw: &str) -> Binder<MapEnvironment> {
    Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
}

fn fallback_environments() -> [MapEnvironment; 2] {
    [
        MapEnvironment::new(),
        MapEnvironment::from_pairs([("VALUE", "")]),
    ]
}

#[test]
fn scalar_helper_rejects_all_non_finite_categories_with_fixed_diagnostics() {
    let result = NON_FINITE_VALUES.map(validators::is_finite());

    assert_eq!(
        result,
        std::array::from_fn(|_| Err(ValidationError::new("value must be finite")))
    );
}

#[test]
fn list_helper_rejects_non_finite_elements_at_every_position() {
    let result = NON_FINITE_VALUES.map(|value| {
        [[value, 1.0, 2.0], [1.0, value, 2.0], [1.0, 2.0, value]]
            .map(|values| validators::all_finite()(&values))
    });

    assert_eq!(
        result,
        std::array::from_fn(|_| std::array::from_fn(|_| Err(ValidationError::new(
            "all list items must be finite"
        ))))
    );
}

#[test]
fn list_helper_accepts_an_empty_typed_list() {
    assert_eq!(validators::all_finite()(&[]), Ok(()));
}

#[test]
fn scalar_policy_rejects_non_finite_input_with_safe_structured_errors() {
    let spec = FloatVar::new("VALUE")
        .sensitive(false)
        .validate(validators::is_finite());
    let result = NON_FINITE.map(|raw| binder(raw).bind(&spec));

    assert_eq!(
        result,
        std::array::from_fn(|_| Err(BindError::validation(
            "VALUE",
            ValidationError::new("value must be finite")
        )))
    );
}

#[test]
fn list_policy_rejects_non_finite_input_with_safe_structured_errors() {
    let spec = ListVar::floats("VALUE")
        .sensitive(false)
        .validate(validators::all_finite());
    let result = NON_FINITE.map(|raw| binder(&format!("1,{raw},2")).bind(&spec));

    assert_eq!(
        result,
        std::array::from_fn(|_| Err(BindError::validation(
            "VALUE",
            ValidationError::new("all list items must be finite")
        )))
    );
}

#[test]
fn scalar_policy_preserves_finite_bits_including_signed_zero_and_subnormals() {
    let spec = FloatVar::new("VALUE").validate(validators::is_finite());
    let result = FINITE.map(|(raw, _)| binder(raw).bind(&spec).map(f64::to_bits));

    assert_eq!(result, FINITE.map(|(_, value)| Ok(value.to_bits())));
}

#[test]
fn list_policy_preserves_finite_bits_including_signed_zero_and_subnormals() {
    let raw = FINITE.map(|(raw, _)| raw).join(", ");
    let result = binder(&raw)
        .bind(&ListVar::floats("VALUE").validate(validators::all_finite()))
        .map(|values| values.into_iter().map(f64::to_bits).collect::<Vec<_>>());

    assert_eq!(
        result,
        Ok(FINITE.map(|(_, value)| value.to_bits()).to_vec())
    );
}

#[test]
fn scalar_parsing_remains_permissive_without_the_policy() {
    let result = NON_FINITE.map(|raw| {
        binder(raw)
            .bind(&FloatVar::new("VALUE"))
            .map(|v| !v.is_finite())
    });

    assert_eq!(result, std::array::from_fn(|_| Ok(true)));
}

#[test]
fn list_parsing_remains_permissive_without_the_policy() {
    let result = NON_FINITE.map(|raw| {
        binder(&format!("1,{raw},2"))
            .bind(&ListVar::floats("VALUE"))
            .map(|values| values.iter().any(|value| !value.is_finite()))
    });

    assert_eq!(result, std::array::from_fn(|_| Ok(true)));
}

#[test]
fn a_lower_bound_alone_still_accepts_positive_infinity() {
    let result = binder("1e999").bind(&FloatVar::new("VALUE").validate(validators::min_value(0.0)));

    assert_eq!(result, Ok(f64::INFINITY));
}

#[test]
fn finite_and_lower_bound_compose_for_rates() {
    let spec = FloatVar::new("VALUE").validate(validators::all_of(vec![
        Box::new(validators::is_finite()),
        Box::new(validators::min_value(0.0)),
    ]));
    let result = ["NaN", "inf", "-inf", "1e999", "-1", "-0", "0", "1e3"].map(|raw| {
        binder(raw)
            .bind(&spec)
            .map(f64::to_bits)
            .map_err(|error| error.error_code())
    });

    assert_eq!(
        result,
        [
            Err("validation_failed"),
            Err("validation_failed"),
            Err("validation_failed"),
            Err("validation_failed"),
            Err("validation_failed"),
            Ok((-0.0_f64).to_bits()),
            Ok(0.0_f64.to_bits()),
            Ok(1000.0_f64.to_bits())
        ]
    );
}

#[test]
fn finite_and_range_compose_for_bounded_timeouts() {
    let spec = FloatVar::new("VALUE")
        .validate(validators::is_finite())
        .validate(validators::in_range(0.0, 300.0));
    let result = ["1e999", "-1", "301", "-0", "1e-3", "300"].map(|raw| {
        binder(raw)
            .bind(&spec)
            .map_err(|error| error.error_code())
            .and_then(|value| {
                std::time::Duration::try_from_secs_f64(value).map_err(|_| "duration_conversion")
            })
    });

    assert_eq!(
        result,
        [
            Err("validation_failed"),
            Err("validation_failed"),
            Err("validation_failed"),
            Ok(std::time::Duration::ZERO),
            Ok(std::time::Duration::from_millis(1)),
            Ok(std::time::Duration::from_secs(300))
        ]
    );
}

#[test]
fn scalar_policy_validates_missing_and_empty_fallbacks_when_enabled() {
    let result = NON_FINITE_VALUES.map(|value| {
        let spec = FloatVar::new("VALUE")
            .default(value)
            .validate_default()
            .validate(validators::is_finite());
        fallback_environments().map(|env| {
            Binder::new(env)
                .bind(&spec)
                .map_err(|error| error.error_code())
        })
    });

    assert_eq!(result, [[Err("validation_failed"); 2]; 3]);
}

#[test]
fn list_policy_validates_missing_and_empty_fallbacks_when_enabled() {
    let result = NON_FINITE_VALUES.map(|value| {
        let spec = ListVar::floats("VALUE")
            .default(vec![1.0, value])
            .validate_default()
            .validate(validators::all_finite());
        fallback_environments().map(|env| {
            Binder::new(env)
                .bind(&spec)
                .map_err(|error| error.error_code())
        })
    });

    assert_eq!(
        result,
        std::array::from_fn(|_| std::array::from_fn(|_| Err("validation_failed")))
    );
}

#[test]
fn scalar_fallbacks_still_skip_validation_by_default() {
    let result = NON_FINITE_VALUES.map(|value| {
        let spec = FloatVar::new("VALUE")
            .default(value)
            .validate(validators::is_finite());
        fallback_environments().map(|env| Binder::new(env).bind(&spec).map(|v| !v.is_finite()))
    });

    assert_eq!(
        result,
        std::array::from_fn(|_| std::array::from_fn(|_| Ok(true)))
    );
}

#[test]
fn list_fallbacks_still_skip_validation_by_default() {
    let result = NON_FINITE_VALUES.map(|value| {
        let spec = ListVar::floats("VALUE")
            .default(vec![value])
            .validate(validators::all_finite());
        fallback_environments().map(|env| {
            Binder::new(env)
                .bind(&spec)
                .map(|values| values.iter().any(|v| !v.is_finite()))
        })
    });

    assert_eq!(
        result,
        std::array::from_fn(|_| std::array::from_fn(|_| Ok(true)))
    );
}

#[test]
fn validated_scalar_fallbacks_preserve_finite_bits() {
    let result = FINITE.map(|(_, value)| {
        Binder::new(MapEnvironment::new())
            .bind(
                &FloatVar::new("VALUE")
                    .default(value)
                    .validate_default()
                    .validate(validators::is_finite()),
            )
            .map(f64::to_bits)
    });

    assert_eq!(result, FINITE.map(|(_, value)| Ok(value.to_bits())));
}

#[test]
fn validated_list_fallbacks_preserve_finite_bits() {
    let values = FINITE.map(|(_, value)| value).to_vec();
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &ListVar::floats("VALUE")
                .default(values)
                .validate_default()
                .validate(validators::all_finite()),
        )
        .map(|values| values.into_iter().map(f64::to_bits).collect::<Vec<_>>());

    assert_eq!(
        result,
        Ok(FINITE.map(|(_, value)| value.to_bits()).to_vec())
    );
}

#[test]
fn optional_scalar_preserves_policy_failures() {
    let result = NON_FINITE.map(|raw| {
        binder(raw)
            .bind(
                &FloatVar::new("VALUE")
                    .validate(validators::is_finite())
                    .optional(),
            )
            .map_err(|error| error.error_code())
    });

    assert_eq!(result, [Err("validation_failed"); 5]);
}

#[test]
fn optional_list_preserves_policy_failures() {
    let result = NON_FINITE.map(|raw| {
        binder(raw)
            .bind(
                &ListVar::floats("VALUE")
                    .validate(validators::all_finite())
                    .optional(),
            )
            .map_err(|error| error.error_code())
    });

    assert_eq!(result, std::array::from_fn(|_| Err("validation_failed")));
}

#[test]
fn optional_scalar_preserves_fallback_validation_failures() {
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &FloatVar::new("VALUE")
                .default(f64::NAN)
                .validate_default()
                .validate(validators::is_finite())
                .optional(),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn optional_list_preserves_fallback_validation_failures() {
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &ListVar::floats("VALUE")
                .default(vec![f64::INFINITY])
                .validate_default()
                .validate(validators::all_finite())
                .optional(),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn scalar_policy_does_not_replace_malformed_input_with_a_fallback() {
    let result = binder("not-a-number")
        .bind(
            &FloatVar::new("VALUE")
                .default(1.0)
                .validate_default()
                .validate(validators::is_finite()),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("parse_variable"));
}

#[test]
fn list_policy_does_not_replace_malformed_input_with_a_fallback() {
    let result = binder("1,not-a-number")
        .bind(
            &ListVar::floats("VALUE")
                .default(vec![1.0])
                .validate_default()
                .validate(validators::all_finite()),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn sensitive_scalar_diagnostics_follow_the_existing_redaction_policy() {
    let error = binder("1e999")
        .bind(&FloatVar::new("VALUE").validate(validators::is_finite()))
        .err();

    assert_eq!(
        error,
        Some(BindError::validation(
            "VALUE",
            ValidationError::new("validation failed for sensitive value")
        ))
    );
}

#[test]
fn sensitive_list_diagnostics_follow_the_existing_redaction_policy() {
    let error = binder("1,1e999")
        .bind(&ListVar::floats("VALUE").validate(validators::all_finite()))
        .err();

    assert_eq!(
        error,
        Some(BindError::validation(
            "VALUE",
            ValidationError::new("validation failed for sensitive value")
        ))
    );
}
